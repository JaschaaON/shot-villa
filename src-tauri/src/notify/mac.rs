//! Mitteilungen über das UserNotifications-Framework. Ein eigener Delegate
//! sorgt dafür, dass sie auch erscheinen, während ein Screeni-Fenster aktiv ist.

use std::path::Path;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{Bool, NSObject, NSObjectProtocol, ProtocolObject};
use objc2::{AnyThread, define_class, msg_send};
use objc2_foundation::{NSBundle, NSError, NSString, NSUUID};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNMutableNotificationContent, UNNotification,
    UNNotificationPresentationOptions, UNNotificationRequest, UNUserNotificationCenter,
    UNUserNotificationCenterDelegate,
};
use tauri::AppHandle;

define_class!(
    // SAFETY: NSObject stellt keine Anforderungen an Unterklassen, kein Drop.
    #[unsafe(super(NSObject))]
    #[name = "ScreeniNotificationDelegate"]
    struct Delegate;

    unsafe impl NSObjectProtocol for Delegate {}

    unsafe impl UNUserNotificationCenterDelegate for Delegate {
        /// Auch im Vordergrund als Banner zeigen.
        #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
        fn will_present(
            &self,
            _center: &UNUserNotificationCenter,
            _notification: &UNNotification,
            handler: &block2::DynBlock<dyn Fn(UNNotificationPresentationOptions)>,
        ) {
            handler
                .call((UNNotificationPresentationOptions::Banner
                    | UNNotificationPresentationOptions::List,));
        }
    }
);

impl Delegate {
    fn new() -> Retained<Self> {
        let this = Self::alloc().set_ivars(());
        // SAFETY: `init` von NSObject.
        unsafe { msg_send![super(this), init] }
    }
}

/// Das Framework funktioniert nur in einem App-Bundle; `tauri dev` startet
/// die nackte Binärdatei.
fn center() -> Option<Retained<UNUserNotificationCenter>> {
    let path = NSBundle::mainBundle().bundlePath().to_string();
    let in_bundle = Path::new(&path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("app"));
    in_bundle.then(UNUserNotificationCenter::currentNotificationCenter)
}

/// Setzt den Delegate und fragt beim ersten Start nach der Erlaubnis.
pub fn init() {
    let Some(center) = center() else {
        log::info!("Mitteilungen nur im App-Bundle verfügbar");
        return;
    };
    let delegate = Delegate::new();
    center.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    // Das Center hält seinen Delegate nur schwach; er lebt so lange wie die App.
    std::mem::forget(delegate);
    let handler = RcBlock::new(|granted: Bool, error: *mut NSError| {
        if !granted.as_bool() {
            // SAFETY: Das Framework übergibt null oder einen gültigen Fehler.
            let reason = unsafe { error.as_ref() }.map_or_else(
                || "abgelehnt".to_owned(),
                |e| e.localizedDescription().to_string(),
            );
            log::info!("Mitteilungen nicht erlaubt: {reason}");
        }
    });
    center.requestAuthorizationWithOptions_completionHandler(
        UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
        &handler,
    );
}

pub fn show(_app: &AppHandle, title: &str, body: &str) {
    let Some(center) = center() else {
        return;
    };
    let content = UNMutableNotificationContent::new();
    content.setTitle(&NSString::from_str(title));
    content.setBody(&NSString::from_str(body));
    let request = UNNotificationRequest::requestWithIdentifier_content_trigger(
        &NSUUID::UUID().UUIDString(),
        &content,
        None,
    );
    let handler = RcBlock::new(|error: *mut NSError| {
        // SAFETY: Das Framework übergibt null oder einen gültigen Fehler.
        if let Some(error) = unsafe { error.as_ref() } {
            log::warn!(
                "Mitteilung fehlgeschlagen: {}",
                error.localizedDescription()
            );
        }
    });
    center.addNotificationRequest_withCompletionHandler(&request, Some(&handler));
}
