//! Text- und QR-Code-Erkennung mit dem Vision-Framework.

use objc2::AnyThread;
use objc2_foundation::{NSArray, NSDictionary};
use objc2_vision::{
    VNDetectBarcodesRequest, VNImageRequestHandler, VNRecognizeTextRequest, VNRequest,
    VNRequestTextRecognitionLevel,
};

use super::{OcrError, Recognition};
use crate::imaging::Image;

/// Vision scheitert gelegentlich beim Laden des Erkennungsmodells (`CRImageReaderError`),
/// besonders in lange laufenden Prozessen. Ein zweiter Versuch genügt oft; bleibt der
/// Fehler, hilft ein Neustart der App.
pub fn recognize(image: &Image) -> Result<Recognition, OcrError> {
    detect(image)
        .or_else(|_| detect(image))
        .map_err(with_restart_hint)
}

fn with_restart_hint(OcrError(message): OcrError) -> OcrError {
    OcrError(format!(
        "{message} Bitte Shot-VILLA beenden und neu starten."
    ))
}

fn detect(image: &Image) -> Result<Recognition, OcrError> {
    let text = VNRecognizeTextRequest::new();
    text.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
    text.setUsesLanguageCorrection(true);
    text.setAutomaticallyDetectsLanguage(true);
    let codes = unsafe { VNDetectBarcodesRequest::new() };

    let handler = unsafe {
        VNImageRequestHandler::initWithCGImage_options(
            VNImageRequestHandler::alloc(),
            image,
            &NSDictionary::new(),
        )
    };
    let requests: [&VNRequest; 2] = [&text, &codes];
    handler
        .performRequests_error(&NSArray::from_slice(&requests))
        .map_err(|error| OcrError(error.localizedDescription().to_string()))?;

    let mut observed: Vec<(f64, f64, String)> = text
        .results()
        .map(|results| {
            results
                .iter()
                .filter_map(|observation| {
                    let candidate = observation.topCandidates(1).firstObject()?;
                    let bounds = unsafe { observation.boundingBox() };
                    // Vision-Koordinaten: normiert, Ursprung unten links.
                    Some((
                        1.0 - bounds.origin.y - bounds.size.height,
                        bounds.origin.x,
                        candidate.string().to_string(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    observed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));

    let codes = unsafe { codes.results() }
        .map(|results| {
            results
                .iter()
                .filter_map(|code| unsafe { code.payloadStringValue() })
                .map(|payload| payload.to_string())
                .collect()
        })
        .unwrap_or_default();

    Ok(Recognition {
        lines: observed.into_iter().map(|(_, _, line)| line).collect(),
        codes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_message_suggests_restart() {
        let error = with_restart_hint(OcrError("Modell fehlt.".into()));
        assert_eq!(
            error.to_string(),
            "Texterkennung fehlgeschlagen: Modell fehlt. Bitte Shot-VILLA beenden und neu starten."
        );
    }
}
