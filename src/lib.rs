pub use session::{ApplicationSession, EndPointSession, Session};
use windows::{
    core::Interface,
    Win32::{
        Media::Audio::{
            eRender, Endpoints::IAudioEndpointVolume, IAudioSessionControl,
            IAudioSessionControl2, IAudioSessionEnumerator, IAudioSessionManager2, IMMDevice,
            IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator,
        },
        System::{
            Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CLSCTX_ALL, COINIT_APARTMENTTHREADED, StructuredStorage::PROPVARIANT, STGM_READ},
            ProcessStatus::K32GetProcessImageFileNameA,
            Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
        }, UI::Shell::PropertiesSystem::{PROPERTYKEY, PropVariantToStringAlloc},
    },
};
use std::{process::exit, sync::Arc};

mod session;

#[derive(Clone)]
pub struct AudioController {
    imm_device_enumerator: Option<IMMDeviceEnumerator>,
    pub audio_devices: Vec<AudioDevice>
}

#[derive(Clone)]
pub struct AudioDevice {
    manufacturer: String,
    device_name: String,
    device: IMMDevice,
    pub sessions: Vec<Arc<dyn Session>>
}

pub enum CoinitMode {
    MultiTreaded,
    ApartmentThreaded
}

impl AudioController {
    pub unsafe fn init(coinit_mode: Option<CoinitMode>) -> Self {
        let mut coinit: windows::Win32::System::Com::COINIT = COINIT_MULTITHREADED;
        if let Some(x) = coinit_mode {
            match x {
                CoinitMode::ApartmentThreaded   => {coinit = COINIT_APARTMENTTHREADED},
                CoinitMode::MultiTreaded        => {coinit = COINIT_MULTITHREADED}
            }
        }
        CoInitializeEx(None, coinit).unwrap_or_else(|err| {
            eprintln!("ERROR: Couldn't initialize windows connection: {err}");
            exit(1);
        });

        Self {
            imm_device_enumerator: None,
            audio_devices: Vec::new()
        }
    }
    pub unsafe fn load_devices(&mut self) {
        self.imm_device_enumerator = Some(
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER).unwrap_or_else(
                |err| {
                    eprintln!("ERROR: Couldn't get Media device enumerator: {err}");
                    exit(1);
                },
            ),
        );

        // Loading the registry property keys for the names and manufacturers for audio devices.
        let device_name_property: *const PROPERTYKEY = &PROPERTYKEY { fmtid:windows::core::GUID::from("a45c254e-df1c-4efd-8020-67d146a850e0"), pid: 2 } as *const PROPERTYKEY;
        let device_manufacturer_property: *const PROPERTYKEY = &PROPERTYKEY { fmtid: windows::core::GUID::from("b3f8fa53-0004-438e-9003-51a46e139bfc"), pid: 6 } as *const PROPERTYKEY;

        // Loop through all active audio devices
        // TODO all unwraps need to be unwarp_or_else
        let active_audio_device_count: u32 = self.imm_device_enumerator.clone().unwrap().EnumAudioEndpoints(eRender, 1).unwrap().GetCount().unwrap();
        for device in 0..active_audio_device_count {
            let imm_audio_device: IMMDevice = self.imm_device_enumerator.clone().unwrap().EnumAudioEndpoints(eRender, 1).unwrap().Item(device).unwrap();

            // Get the registry properties for naming the device
            let manufacturer_property_variant: *mut PROPVARIANT = &mut imm_audio_device.OpenPropertyStore(STGM_READ).unwrap().GetValue(device_manufacturer_property).unwrap();
            let manufacturer_name: String = PropVariantToStringAlloc(manufacturer_property_variant).unwrap().to_string().unwrap();

            let device_name_property_variant: *mut PROPVARIANT = &mut imm_audio_device.OpenPropertyStore(STGM_READ).unwrap().GetValue(device_name_property).unwrap();
            let device_name: String = PropVariantToStringAlloc(device_name_property_variant).unwrap().to_string().unwrap();

            // Create the device structure and add it to the list of devices
            let audio_device: AudioDevice = AudioDevice { manufacturer: (manufacturer_name), device_name: (device_name), device: (imm_audio_device.clone()), sessions: Vec::new() };

            self.audio_devices.push(audio_device);
        }
    }

    pub unsafe fn load_all_sessions(mut self) -> AudioController
    {
        let mut new_audio_devices: Vec<AudioDevice> = Vec::new();
        for audio_device in self.audio_devices {
            let new_audio_device = Self::load_sessions(audio_device);
            new_audio_devices.push(new_audio_device)
        }

        self.audio_devices = new_audio_devices;
        return self
    }

    // Loads all of the sesssions for a given AudioDevice
    pub unsafe fn load_sessions(mut audio_device:AudioDevice) -> AudioDevice
    {
        // Getting master volume
        let simple_audio_volume: IAudioEndpointVolume = audio_device.device
            .clone()
            .Activate(CLSCTX_ALL, None)
            .unwrap_or_else(|err|{
                eprintln!("ERROR: Couldn't get Endpoint volume control: {err}");
                exit(1);
            });

        audio_device.sessions.push(Arc::new(EndPointSession::new(simple_audio_volume, "master".to_string())));

        // Getting program volumes
        let session_manager2: IAudioSessionManager2 = audio_device.device
            .clone()
            .Activate(CLSCTX_INPROC_SERVER, None)
            .unwrap_or_else(|err| {
                eprintln!("ERROR: Couldnt get AudioSessionManager for enumerating over processes... {err}");
                exit(1);
            });

        let session_enumerator: IAudioSessionEnumerator = session_manager2
            .GetSessionEnumerator()
            .unwrap_or_else(|err| {
                eprintln!("ERROR: Couldnt get session enumerator... {err}");
                exit(1);
            });

        for i in 0..session_enumerator.GetCount().unwrap() {
            let normal_session_control: Option<IAudioSessionControl> =
                session_enumerator.GetSession(i).ok();
            if normal_session_control.is_none() {
                eprintln!("ERROR: Couldn't get session control of audio session...");
                continue;
            }

            let session_control: Option<IAudioSessionControl2> =
                normal_session_control.unwrap().cast().ok();
            if session_control.is_none() {
                eprintln!(
                    "ERROR: Couldn't convert from normal session control to session control 2"
                );
                continue;
            }

            let pid = session_control.as_ref().unwrap().GetProcessId().unwrap();
            if pid == 0 {
                continue;
            }
            let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid).ok();
            if process.is_none() {
                eprintln!("ERROR: Couldn't get process information of process id {pid}");
                continue;
            }
            let mut filename: [u8; 128] = [0; 128];
            K32GetProcessImageFileNameA(process, &mut filename);
            let mut new_filename: Vec<u8> = vec![];
            for i in filename.iter() {
                if i == &(0 as u8) {
                    continue;
                }
                new_filename.push(i.clone());
            }
            let mut str_filename = match String::from_utf8(new_filename) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("ERROR: Filename couldn't be converted to string, {err}");
                    continue;
                }
            };
            str_filename = match str_filename.split("\\").last() {
                Some(data) => data.to_string().replace(".exe", ""),
                None => {
                    continue;
                }
            };
            let audio_control: ISimpleAudioVolume = match session_control.unwrap().cast() {
                Ok(data) => data,
                Err(err) => {
                    eprintln!(
                        "ERROR: Couldn't get the simpleaudiovolume from session controller: {err}"
                    );
                    continue;
                }
            };
            let application_session = ApplicationSession::new(audio_control, str_filename);
            audio_device.sessions.push(Arc::new(application_session));
        }

        return audio_device;
        
    }

    pub unsafe fn get_all_audio_device_names(&self) -> Vec<String> {
        self.audio_devices.iter().map(|i| i.get_name()).collect()
    }

}

impl AudioDevice{
    pub unsafe fn get_all_session_names(&self) -> Vec<String> {
        self.sessions.iter().map(|i| i.get_name()).collect()
    }

    pub unsafe fn get_session_by_name(&self, name: String) -> Option<&Arc<dyn Session>> {
        self.sessions.iter().find(|i| i.get_name() == name)
    }

    pub unsafe fn get_name(&self) -> String {
        let seperator = " - ".to_string();
        return self.manufacturer.clone() + &seperator + &self.device_name.clone();
        
    }
}
