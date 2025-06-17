use std::sync::RwLock;
use crate::structs::{EqualiserChannelSettings, EqualiserSettings, VisualiserSettings, VisualiserType};



pub static IS_CAPTURE_RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub static EQUALISER_CONFIG: RwLock<EqualiserSettings> = RwLock::new(EqualiserSettings(
    EqualiserChannelSettings {
        preamp: 0.0,
        channelLeft: true,
        bassGain: 0.0,
        lowGain: 0.0,
        midGain: 0.0,
        highGain: 0.0,
        trebleGain: 0.0,
    }, 
    EqualiserChannelSettings {
        preamp: 0.0,
        channelLeft: false,
        bassGain: 0.0,
        lowGain: 0.0,
        midGain: 0.0,
        highGain: 0.0,
        trebleGain: 0.0,
    }
));

pub static VISUALISER_CONFIG: RwLock<VisualiserSettings> = RwLock::new(VisualiserSettings {
    barsColour: (0, 0, 0, 170),
    visualiserType: VisualiserType::Linear1,
    useDesktopBackground: true,
    resolution: 128,
});
