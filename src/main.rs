use cpal::{traits::HostTrait, traits::StreamTrait};
use cpal::traits::DeviceTrait;
use std::time::{Duration};
use std::thread;
use std::sync::{Arc, Mutex};
use cocoa::base::nil;
use cocoa::foundation::NSString;
use cocoa::appkit::{ NSApp,
    NSApplication,
    NSApplicationActivationPolicyProhibited,
    NSStatusBar,
    NSVariableStatusItemLength,
    NSStatusItem,
    NSButton };


fn init_cocoa() -> (cocoa::base::id, cocoa::base::id) {
    let app = unsafe { NSApp() };

    let button = unsafe {
        app.setActivationPolicy_(NSApplicationActivationPolicyProhibited);
        let status_item = NSStatusBar::systemStatusBar(nil).statusItemWithLength_(NSVariableStatusItemLength);
        status_item.button()
    };

    return (app, button);
}

fn main() {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device");
    let config = device
        .default_input_config().expect("Failed to get default input config");
        
    const MAX_LENGTH: usize = 50;
    let sound_maxs = Arc::new(Mutex::new([0.; MAX_LENGTH]));
    let mut index:usize = 0;
    
    let sm_access = sound_maxs.clone();
    let stream = device.build_input_stream(
        &config.into(),
        move |data, _: &_| {
            let mut max_sounds = sm_access.lock().unwrap();
            max_sounds[index] = data.iter().fold(f32::NEG_INFINITY, |acc, &x| acc.max(x));
            drop(max_sounds);
            index += 1;
            index %= MAX_LENGTH;
        },
        move |err| {
            eprintln!("Error occurred in the stream: {}", err);
        },
    ).expect("Failed to build input stream.");

    let sm_access = sound_maxs.clone();

    let (app, button) = init_cocoa();

    let button_ptr = button as u64;
    thread::spawn(move || {
        let btn = button_ptr as cocoa::base::id;

        loop {
            let max_sounds = sm_access.lock().unwrap();
            let average: f32 = max_sounds.iter().sum::<f32>() / MAX_LENGTH as f32;            
            drop(max_sounds);
            unsafe{ btn.setTitle_(NSString::alloc(nil).init_str(&format!("{:.0}", 100.0 * average))) }
            thread::sleep(Duration::from_millis(200));
        }
    });
    stream.play().expect("Failed to play stream.");
    unsafe { app.run(); }
    drop(stream);
}

