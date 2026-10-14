//! Text- und QR-Code-Erkennung mit dem Vision-Framework.

use objc2::AnyThread;
use objc2_foundation::{NSArray, NSDictionary};
use objc2_vision::{
    VNDetectBarcodesRequest, VNImageRequestHandler, VNRecognizeTextRequest, VNRequest,
    VNRequestTextRecognitionLevel,
};

use super::{OcrError, Recognition};
use crate::imaging::Image;

pub fn recognize(image: &Image) -> Result<Recognition, OcrError> {
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
