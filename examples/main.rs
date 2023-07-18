use std::sync::Arc;

use windows_volume_control::{AudioController, AudioDevice, Session};
use std::convert::TryFrom;

fn main() {
    unsafe {
        let mut controller = AudioController::init(None);
        controller.load_devices();
        controller = controller.load_all_sessions();


        println!("Hello User!");

        let mut state = 0;
        let mut selected_device: Option<AudioDevice> = None;
        let mut selected_session: Option<Arc<dyn Session>> = None;

        loop
        {
            if state == 0 // Selecting the audio device
            {
                println!("Here is a list of all your audio devices:");
                let mut n = 0;
                for audio_device in controller.audio_devices.clone()
                {
                    let name = audio_device.get_name();
                    println!("{:?}: {:?}", n, name);
                    n += 1;
                }

                let mut line = String::new();
                println!("Please select one of the above by entering it's number. If you wish to exit the program, press \'ctrl + c\'.");
                loop
                {
                    let b1 = std::io::stdin().read_line(&mut line).unwrap();
                    if b1 == 0
                    {
                        continue;
                    }

                    let selection: i32 = line.trim().parse()
                        .unwrap_or_else(|err|{
                            println!("Not a number: {err}");
                            -1
                        });
                    
                    if selection < 0 || selection > n
                    {
                        println!("Invalid input. Please input a number between 0 and {:?}.", &n);
                        continue;
                    }

                    selected_device = Some(controller.audio_devices[usize::try_from(selection).unwrap()].clone());
                    state = 1;
                    break;
                }
            }
            else if state == 1 // Selecting the session
            {
                if selected_device.clone().is_none()
                {
                    println!("Error, somehow in impossible state. Fixing!");
                    state -= 1;
                    continue;
                }

                println!("The following is a list of audio channels on the {:?} device:", selected_device.clone().unwrap().get_name());
                let mut n = 0;
                for audio_session in selected_device.clone().unwrap().sessions
                {
                    let name = audio_session.get_name();
                    println!("{:?}: {:?}", n, name);
                    n += 1;
                }

                let mut line = String::new();
                println!("Please select one of the above by entering it's number. If you wish to exit the program, press \'ctrl + c\'. If you wish to go back, enter 'b'.");
                loop
                {
                    let b1 = std::io::stdin().read_line(&mut line).unwrap();
                    if b1 == 0
                    {
                        continue;
                    }

                    if line.trim() == "b"
                    {
                        state -= 1;
                        selected_device = None;
                        break;
                    }

                    let selection: i32 = line.trim().parse()
                        .unwrap_or_else(|err|{
                            println!("Not a number: {err}");
                            -1
                        });
                    
                    if selection < 0 || selection > n
                    {
                        println!("Invalid input. Please input a number between 0 and {:?}.", &n);
                        continue;
                    }

                    selected_session = Some(selected_device.clone().unwrap().sessions[usize::try_from(selection).unwrap()].clone());
                    state = 2;
                    break;
                }
                
            }
            else if state == 2 // Setting the volume
            {
                if selected_session.clone().is_none()
                {
                    println!("Error, somehow in impossible state. Fixing!");
                    state -= 1;
                    continue;
                }
                println!("Please set a volume for this channel (0.0 - 100.0). If you wish to go back, enter 'b'.");

                let mut line = String::new();
                loop
                {
                    let b1 = std::io::stdin().read_line(&mut line).unwrap();
                    if b1 == 0
                    {
                        continue;
                    }

                    if line.trim() == "b"
                    {
                        state -= 1;
                        selected_session = None;
                        break;
                    }

                    let selection: i32 = line.trim().parse()
                        .unwrap_or_else(|err|{
                            println!("Not a number: {err}");
                            -1
                        });
                    
                    if selection < 0 || selection > 100
                    {
                        println!("Invalid input. Please input a number between 0.0 and 100.0");
                        continue;
                    }

                    selected_session.clone().unwrap().set_volume(selection as f32 / 100.0);  //Inputs into the function must be between 0 and 1
                    println!("Volume has been set to {:?}", selection);
                    selected_session = None;
                    state = 1;
                    break;
                }
            }
            else
            {
                println!("You've broken the loop somehow. Returning to start.");
                state = 0;
            }
        }

        //let sessions = audio_device.get_all_session_names();
        //for session in sessions {
        //    println!("- {:?}", session)
        //}


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

