use i3ipc::{
    event::{inner::WindowChange, Event, WindowEventInfo},
    reply::Node,
    I3EventListener, Subscription,
};
use std::collections::HashMap;
use xcb::{
    x::ModMask,
    xkb::{self, DeviceSpec, Group},
    Extension,
};

struct Keyboard {
    con: xcb::Connection,
}

impl Keyboard {
    fn new() -> Keyboard {
        let (con, _screen_num) =
            xcb::Connection::connect_with_extensions(None, &[Extension::Xkb], &[]).unwrap();

        assert!(con
            .wait_for_reply(con.send_request(&xkb::UseExtension {
                wanted_major: 1,
                wanted_minor: 0,
            }))
            .unwrap()
            .supported());

        Keyboard { con }
    }

    fn group(&self) -> Group {
        self.con
            .wait_for_reply(self.con.send_request(&xkb::GetState {
                device_spec: xkb::Id::UseCoreKbd as DeviceSpec,
            }))
            .unwrap()
            .group()
    }

    fn set_group(&self, group: Group) {
        self.con
            .send_and_check_request(&xkb::LatchLockState {
                device_spec: xkb::Id::UseCoreKbd as DeviceSpec,
                affect_mod_locks: ModMask::empty(),
                mod_locks: ModMask::empty(),
                lock_group: true,
                group_lock: group,
                affect_mod_latches: ModMask::empty(),
                latch_group: false,
                group_latch: 0,
            })
            .unwrap();
    }
}

fn main() {
    let kb: Keyboard = Keyboard::new();
    let mut i3sock = I3EventListener::connect().unwrap();
    i3sock.subscribe(&[Subscription::Window]).unwrap();

    let mut current = 0;
    let mut wins = HashMap::new();

    for event in i3sock.listen() {
        match event {
            Ok(Event::WindowEvent(WindowEventInfo {
                change,
                container: Node {
                    window: Some(win), ..
                },
            })) => match change {
                WindowChange::Close => {
                    wins.remove(&win);
                }
                WindowChange::Focus if win != current => {
                    if let Some(group) = wins.get_mut(&current) {
                        *group = kb.group();
                    }

                    current = win;
                    kb.set_group(*wins.entry(win).or_insert(Group::N1));
                }
                _ => {}
            },
            Err(err) => {
                eprintln!("Error: {err}");
                break;
            }
            _ => {}
        };
    }
}
