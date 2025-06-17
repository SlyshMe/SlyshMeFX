use tauri::Manager;



#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct AppConfig {
    pub visualiserSettings: VisualiserSettings,
    pub equaliserSettings: EqualiserSettings,
}
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            visualiserSettings: VisualiserSettings::default(),
            equaliserSettings: EqualiserSettings::default(),
        }
    }
}
impl AppConfig {
    pub fn save(&self, app: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        let configDir = app.path().app_local_data_dir()?;
        let configPath = configDir.join("config.json");
        let _ = std::fs::create_dir_all(&configDir);

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(configPath, json)?;

        Ok(())
    }
    
    pub fn load(app: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let configDir = app.path().app_local_data_dir()?;
        let configPath = configDir.join("config.json");
        
        if !configPath.exists() {
            return Ok(Self::default())
        }

        let json = std::fs::read_to_string(configPath)?;
        let config = serde_json::from_str(&json)?;
        Ok(config)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct EqualiserChannelSettings {
    pub preamp: f32,
    pub channelLeft: bool,
    pub bassGain: f32,
    pub lowGain: f32,
    pub midGain: f32,
    pub highGain: f32,
    pub trebleGain: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct EqualiserSettings(pub EqualiserChannelSettings, pub EqualiserChannelSettings); // 0 is left, 1 is right
impl Default for EqualiserSettings {
    fn default() -> Self {
        Self(
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
        )
    }
}
impl EqualiserSettings {
    pub fn toConfig(&self) -> String {
        format!(
            "Preamp: {} dB\n\nFilter: ON LS Fc 250 Hz Gain {}dB\nFilter: ON PK Fc 500 Hz Gain {}dB Q 2.0\nFilter: ON PK Fc 750 Hz Gain {}dB Q 3.0\nFilter: ON PK Fc 1000 Hz Gain {}dB Q 4.0\nFilter: ON HS Fc 1250 Hz Gain {}dB",
            self.0.preamp, self.0.bassGain, self.0.lowGain, self.0.midGain, self.0.highGain, self.0.trebleGain
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FrequencyInterval {
    pub index: u16,
    pub volume: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct VisualiserSettings {
    pub barsColour: (u8, u8, u8, u8), // rgba, 0-255
    pub visualiserType: VisualiserType,
    pub useDesktopBackground: bool,
    pub resolution: u16,
}
impl Default for VisualiserSettings {
    fn default() -> Self {
        Self {
            barsColour: (0, 0, 0, 170),
            visualiserType: VisualiserType::Linear1,
            useDesktopBackground: true,
            resolution: 128,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub enum VisualiserType {
    Linear1, // linear by frequency
    Linear2, // linear by amount of separate frequencies
    Log, // normal logarithmic
}
