use std::time::Duration;

use windows::Win32::{Media::Audio::{eRender, eMultimedia}, System::Com::{STGM_READ, StructuredStorage::PROPVARIANT}, UI::Shell::PropertiesSystem::{PROPERTYKEY, PropVariantToStringAlloc}};
use windows_volume_control::AudioController;

fn main() {
    unsafe {
        let mut controller = AudioController::init(None);
        controller.load_devices();
        controller = controller.load_all_sessions();

        for audio_device in controller.audio_devices{
            let name = audio_device.get_name();
            let sessions = audio_device.get_all_session_names();

            println!("Device: {:?}", name);
            for session in sessions {
                println!("- {:?}", session)
            }
            println!("");
        }

        // controller.GetDefaultAudioEnpointVolumeControl();
        // controller.GetAllProcessSessions();
        // let test = controller.get_all_session_names();
        // let master_session = controller.get_session_by_name("master".to_string());
        //println!("{:?}",master_session.unwrap().getVolume());

        // The line below gets all of the Audio endpoints. the "dwstatemask" value can be from 1-4.
        // 1: Active
        // 2: Disabled
        // 3: Not Present
        // 4: Unplugged
        // controller.imm_device_enumerator.unwrap().EnumAudioEndpoints(eRender, 1).getCount();

        // This is the propety key for the name of the audio device.
        // let device_name_property: PROPERTYKEY = PROPERTYKEY { fmtid: windows::core::GUID::from("a45c254e-df1c-4efd-8020-67d146a850e0"), pid: 2};

        // This is the property key for the name of the audio device manufacturer.
        // let device_manufacturer_name_property: PROPERTYKEY = PROPERTYKEY { fmtid: windows::core::GUID::from("b3f8fa53-0004-438e-9003-51a46e139bfc"), pid: 6};

        // This library is a little insane, so we need to manipulate it slightly
        // let device_manufacturer_name_property_raw = &device_manufacturer_name_property as *const PROPERTYKEY;

        // This is the value, but it's a pointer to memory in windows... kinda?
        // let mut property_variant = controller.default_device.unwrap().OpenPropertyStore(STGM_READ).unwrap().GetValue(device_manufacturer_name_property_raw).unwrap();

        // We once again need to manipulate the value a bit to fit into the next function
        // let property_variant_raw = &mut property_variant as *mut PROPVARIANT;

        // This gets the actual string at the pointer
        // let device_manufacturer_name = PropVariantToStringAlloc(property_variant_raw).unwrap().to_string().unwrap();
        
        // println!("{:?}",device_manufacturer_name);
    }
}
