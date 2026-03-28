use super::{PasteModifier, PlatformProvider};

pub struct Platform;

impl PlatformProvider for Platform {
    fn paste_modifier() -> PasteModifier {
        PasteModifier::Super
    }
}
