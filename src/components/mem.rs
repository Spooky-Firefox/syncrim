use crate::common::{
    Component, Id, Input, InputPort, OutputType, Ports, Signal, SignalSigned, SignalUnsigned,
    Simulator,
};
use log::*;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, convert::TryFrom};

#[derive(Serialize, Deserialize)]
pub struct Mem {
    pub id: Id,
    pub pos: (f32, f32),
    pub width: f32,
    pub height: f32,

    // configuration
    pub big_endian: bool,

    // ports
    pub data: InputPort,
    pub addr: InputPort,
    pub ctrl: InputPort,
    pub sign: InputPort,
    pub size: InputPort,

    // memory
    pub memory: Memory,
    // later history... tbd
    #[cfg(feature = "gui-egui")]
    #[serde(skip)]
    pub egui_x: crate::common::EguiExtra,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Memory {
    bytes: RefCell<HashMap<usize, u8>>,
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Mem {
    pub fn new(
        id: &str,
        pos: (f32, f32),
        width: f32,
        height: f32,
        big_endian: bool,
        data: Input,
        addr: Input,
        ctrl: Input,
        sign: Input,
        size: Input,
        memory: Memory,
    ) -> Self {
        Mem {
            id: id.to_string(),
            pos,
            width: 0f32,
            height: 0f32,
            big_endian,
            data: InputPort {
                port_id: String::from("data"),
                input: data,
            },
            addr: InputPort {
                port_id: String::from("addr"),
                input: addr,
            },
            ctrl: InputPort {
                port_id: String::from("ctrl"),
                input: ctrl,
            },
            sign: InputPort {
                port_id: String::from("sign"),
                input: sign,
            },
            size: InputPort {
                port_id: String::from("size"),
                input: size,
            },
            memory,
            #[cfg(feature = "gui-egui")]
            egui_x: crate::common::EguiExtra::default(),
        }
    }
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            bytes: RefCell::new(HashMap::new()),
        }
    }

    fn align(&self, addr: usize, size: usize) -> Signal {
        Signal::Data((addr % size != 0) as SignalUnsigned)
    }

    fn read(&self, addr: usize, size: usize, sign: bool, big_endian: bool) -> Signal {
        let data: Vec<u8> = (0..size)
            .map(|i| *self.bytes.borrow().get(&(addr + i)).unwrap_or(&0))
            .collect();

        let data = data.as_slice();

        trace!("{:x?}", data);

        Signal::Data(match size {
            1 => {
                if sign {
                    data[0] as i8 as SignalSigned as SignalUnsigned
                } else {
                    data[0] as SignalUnsigned
                }
            }
            2 => {
                if sign {
                    if big_endian {
                        trace!("read signed half word be");
                        let i_16 = i16::from_be_bytes(data.try_into().unwrap());
                        trace!("i_16 {:x?}", i_16);
                        let i_32 = i_16 as i32;
                        trace!("i_32 {:x?}", i_32);
                        i_32 as SignalUnsigned
                    } else {
                        trace!("read signed half word le");
                        let i_16 = i16::from_le_bytes(data.try_into().unwrap());
                        trace!("i_16 {:x?}", i_16);
                        let i_32 = i_16 as i32;
                        trace!("i_32 {:x?}", i_32);
                        i_32 as SignalUnsigned
                    }
                } else if big_endian {
                    trace!("read unsigned half word be");
                    let u_16 = u16::from_be_bytes(data.try_into().unwrap());
                    trace!("u_16 {:x?}", u_16);
                    let u_32 = u_16 as u32;
                    trace!("u_32 {:x?}", u_32);
                    u_32 as SignalUnsigned
                } else {
                    trace!("read unsigned half word le");
                    let u_16 = u16::from_le_bytes(data.try_into().unwrap());
                    trace!("u_16 {:x?}", u_16);
                    let u_32 = u_16 as u32;
                    trace!("u_32 {:x?}", u_32);
                    u_32 as SignalUnsigned
                }
            }
            4 => {
                if sign {
                    if big_endian {
                        i32::from_be_bytes(data.try_into().unwrap()) as SignalUnsigned
                    } else {
                        i32::from_le_bytes(data.try_into().unwrap()) as SignalUnsigned
                    }
                } else if big_endian {
                    u32::from_be_bytes(data.try_into().unwrap()) as SignalUnsigned
                } else {
                    u32::from_le_bytes(data.try_into().unwrap()) as SignalUnsigned
                }
            }
            _ => panic!("illegal sized memory operation"),
        })
    }

    fn write(&self, addr: usize, size: usize, big_endian: bool, data: Signal) {
        let data: SignalUnsigned = data.try_into().unwrap();
        match size {
            1 => {
                trace!("write byte");
                self.bytes.borrow_mut().insert(addr, data as u8);
            }
            2 => {
                if big_endian {
                    trace!("write half word be");
                    (data as u16)
                        .to_be_bytes()
                        .iter()
                        .enumerate()
                        .for_each(|(i, bytes)| {
                            self.bytes.borrow_mut().insert(addr + i, *bytes);
                        })
                } else {
                    trace!("write half word le");
                    (data as u16)
                        .to_le_bytes()
                        .iter()
                        .enumerate()
                        .for_each(|(i, bytes)| {
                            self.bytes.borrow_mut().insert(addr + i, *bytes);
                        })
                }
            }

            4 => {
                if big_endian {
                    trace!("write word be");
                    data.to_be_bytes()
                        .iter()
                        .enumerate()
                        .for_each(|(i, bytes)| {
                            self.bytes.borrow_mut().insert(addr + i, *bytes);
                        })
                } else {
                    trace!("write word le");
                    data.to_le_bytes()
                        .iter()
                        .enumerate()
                        .for_each(|(i, bytes)| {
                            self.bytes.borrow_mut().insert(addr + i, *bytes);
                        })
                }
            }
            _ => {
                panic!("illegal sized memory operation, size = {}", size)
            }
        };
    }
}

#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)] // Unfortunately Rust does not allow Signal here, we need to cast manually
pub enum MemCtrl {
    None,
    Read,
    Write,
}

#[typetag::serde()]
impl Component for Mem {
    fn to_(&self) {
        trace!("Mem");
    }

    fn get_id_ports(&self) -> (Id, Ports) {
        (
            self.id.clone(),
            Ports::new(
                vec![&self.data, &self.addr, &self.ctrl],
                OutputType::Combinatorial,
                vec!["data", "err"],
            ),
        )
    }

    fn clock(&self, simulator: &mut Simulator) {
        let data = simulator.get_input_val(&self.data.input);
        let addr: SignalUnsigned = simulator
            .get_input_val(&self.addr.input)
            .try_into()
            .unwrap();
        let addr = addr as usize;
        let ctrl: SignalUnsigned = simulator
            .get_input_val(&self.ctrl.input)
            .try_into()
            .unwrap();
        let ctrl = MemCtrl::try_from(ctrl as u8).unwrap();
        let size: SignalUnsigned = simulator
            .get_input_val(&self.size.input)
            .try_into()
            .unwrap();
        let size = size as usize;
        let sign: SignalUnsigned = simulator
            .get_input_val(&self.sign.input)
            .try_into()
            .unwrap();
        let sign = sign != 0;

        match ctrl {
            MemCtrl::Read => {
                trace!("read addr {:?} size {:?}", addr, size);
                let value = self.memory.read(addr, size, sign, self.big_endian);
                simulator.set_out_val(&self.id, "data", value);
                let value = self.memory.align(addr, size);
                trace!("align {:?}", value);
                simulator.set_out_val(&self.id, "err", value); // align
            }
            MemCtrl::Write => {
                trace!("write addr {:?} size {:?}", addr, size);
                self.memory.write(addr, size, self.big_endian, data);
                let value = self.memory.align(addr, size);
                trace!("align {:?}", value);
                simulator.set_out_val(&self.id, "err", value); // align
            }
            MemCtrl::None => {
                trace!("no read/write");
            }
        }

        trace!("memory {:?}", self.memory);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::ComponentStore;
    use crate::components::ProbeOut;
    use std::rc::Rc;

    #[test]
    fn test_mem_be() {
        let cs = ComponentStore {
            store: vec![
                Rc::new(ProbeOut::new("data")),
                Rc::new(ProbeOut::new("addr")),
                Rc::new(ProbeOut::new("ctrl")),
                Rc::new(ProbeOut::new("size")),
                Rc::new(ProbeOut::new("sign")),
                Rc::new(Mem::new(
                    "mem",
                    (0.0, 0.0),
                    0.0,
                    0.0,
                    // configuration
                    true, // i.e., big endian
                    // ports
                    Input::new("data", "out"),
                    Input::new("addr", "out"),
                    Input::new("ctrl", "out"),
                    Input::new("size", "out"),
                    Input::new("sign", "out"),
                    // memory
                    Memory {
                        bytes: RefCell::new(HashMap::new()),
                    },
                )),
            ],
        };

        let mut clock = 0;
        let mut simulator = Simulator::new(cs, &mut clock);

        assert_eq!(clock, 1);

        // outputs
        let out = &Input::new("mem", "data");
        let err = &Input::new("mem", "err");

        // reset
        assert_eq!(simulator.get_input_val(out), 0.into());
        assert_eq!(
            simulator.get_input_val(err),
            (false as SignalUnsigned).into()
        );

        println!("<setup for write 42 to addr 4>");

        simulator.set_out_val("data", "out", 0xf0);
        simulator.set_out_val("addr", "out", 4);
        simulator.set_out_val("ctrl", "out", MemCtrl::Write as SignalUnsigned);
        simulator.set_out_val("size", "out", 1);
        println!("sim_state {:?}", simulator.sim_state);

        println!("<clock>");
        simulator.clock(&mut clock);
        println!("sim_state {:?}", simulator.sim_state);

        assert_eq!(clock, 2);
        assert_eq!(simulator.get_input_val(out), 0.into());
        assert_eq!(
            simulator.get_input_val(err),
            (false as SignalUnsigned).into()
        );

        println!("<setup for read byte from addr 4>");

        simulator.set_out_val("ctrl", "out", MemCtrl::Read as SignalUnsigned);
        simulator.set_out_val("size", "out", 1);

        simulator.clock(&mut clock);

        assert_eq!(clock, 3);
        assert_eq!(simulator.get_input_val(out), 0xf0.into());
        assert_eq!(
            simulator.get_input_val(err),
            (false as SignalUnsigned).into()
        );

        println!("<setup for read byte from addr 4>");
        simulator.set_out_val("size", "out", 1);
        simulator.set_out_val("sign", "out", true);

        simulator.clock(&mut clock);
        assert_eq!(clock, 4);
        assert_eq!(simulator.get_input_val(out), 0xffff_fff0.into());
        assert_eq!(
            simulator.get_input_val(err),
            (false as SignalUnsigned).into()
        );

        println!("<setup for read half-word from addr 4>");
        simulator.set_out_val("size", "out", 2);
        simulator.set_out_val("sign", "out", true as SignalUnsigned);

        simulator.clock(&mut clock);
        assert_eq!(clock, 5);
        assert_eq!(simulator.get_input_val(out), 0xffff_f000.into());
        assert_eq!(
            simulator.get_input_val(err),
            (false as SignalUnsigned).into()
        );

        println!("<setup for read word from addr 4>");
        simulator.set_out_val("size", "out", 4);
        simulator.set_out_val("sign", "out", true);

        simulator.clock(&mut clock);
        assert_eq!(clock, 6);
        assert_eq!(simulator.get_input_val(out), 0xf000_0000.into());
        assert_eq!(simulator.get_input_val(err), false.into());

        println!("<setup for read word from addr 5>");
        simulator.set_out_val("addr", "out", 5);

        simulator.clock(&mut clock);
        assert_eq!(clock, 7);
        assert_eq!(simulator.get_input_val(err), true.into());

        println!("<setup for read word from addr 6>");
        simulator.set_out_val("addr", "out", 6);

        simulator.clock(&mut clock);
        assert_eq!(clock, 8);
        assert_eq!(simulator.get_input_val(err), true.into());

        println!("<setup for read word from addr 7>");
        simulator.set_out_val("addr", "out", 7);

        simulator.clock(&mut clock);
        assert_eq!(clock, 9);
        assert_eq!(simulator.get_input_val(err), true.into());

        println!("<setup for read word from addr 8>");
        simulator.set_out_val("addr", "out", 8);

        simulator.clock(&mut clock);
        assert_eq!(clock, 10);
        assert_eq!(simulator.get_input_val(err), false.into());

        println!("<setup for read half-word from addr 9>");
        simulator.set_out_val("addr", "out", 9);
        simulator.set_out_val("size", "out", 2);
        simulator.clock(&mut clock);
        assert_eq!(clock, 11);
        assert_eq!(simulator.get_input_val(err), true.into());

        println!("<setup for read half-word from addr 10>");
        simulator.set_out_val("addr", "out", 10);

        simulator.clock(&mut clock);
        assert_eq!(clock, 12);
        assert_eq!(simulator.get_input_val(err), false.into());

        println!("<setup for write half-word at add 10>");
        simulator.set_out_val("addr", "out", 10);
        simulator.set_out_val("data", "out", 0x1234);
        simulator.set_out_val("ctrl", "out", MemCtrl::Write as SignalUnsigned);
        simulator.clock(&mut clock);
        assert_eq!(clock, 13);
        assert_eq!(simulator.get_input_val(err), false.into());

        println!("<setup for read byte at add 10>");
        simulator.set_out_val("ctrl", "out", MemCtrl::Read as SignalUnsigned);
        simulator.set_out_val("size", "out", 1);
        simulator.clock(&mut clock);
        assert_eq!(clock, 14);
        assert_eq!(simulator.get_input_val(out), 0x12.into());

        println!("<setup for read byte at add 11>");
        simulator.set_out_val("addr", "out", 11);
        simulator.clock(&mut clock);
        assert_eq!(clock, 15);
        assert_eq!(simulator.get_input_val(out), 0x34.into());

        println!("test done")
    }

    #[test]
    fn test_mem_le() {
        let cs = ComponentStore {
            store: vec![
                Rc::new(ProbeOut::new("data")),
                Rc::new(ProbeOut::new("addr")),
                Rc::new(ProbeOut::new("ctrl")),
                Rc::new(ProbeOut::new("size")),
                Rc::new(ProbeOut::new("sign")),
                Rc::new(Mem::new(
                    "mem".into(),
                    (0.0, 0.0),
                    0.0,
                    0.0,
                    // configuration
                    false, // i.e., little endian
                    // ports
                    Input::new("data", "out"),
                    Input::new("addr", "out"),
                    Input::new("ctrl", "out"),
                    Input::new("size", "out"),
                    Input::new("sign", "out"),
                    // memory
                    Memory {
                        bytes: RefCell::new(HashMap::new()),
                    },
                    // later history... tbd
                )),
            ],
        };

        let mut clock = 0;
        let mut simulator = Simulator::new(cs, &mut clock);

        assert_eq!(clock, 1);

        // outputs
        let out = &Input::new("mem", "data");
        let err = &Input::new("mem", "err");

        // reset
        assert_eq!(simulator.get_input_val(out), 0.into());
        assert_eq!(simulator.get_input_val(err), false.into());

        // println!("<setup for write 42 to addr 4>");

        simulator.set_out_val("data", "out", 0xf0);
        simulator.set_out_val("addr", "out", 4);
        simulator.set_out_val("ctrl", "out", MemCtrl::Write as SignalUnsigned);
        simulator.set_out_val("size", "out", 1); // byte

        println!("sim_state {:?}", simulator.sim_state);

        println!("<clock>");
        simulator.clock(&mut clock);
        println!("sim_state {:?}", simulator.sim_state);

        assert_eq!(clock, 2);
        assert_eq!(simulator.get_input_val(out), 0.into());
        assert_eq!(simulator.get_input_val(err), false.into());

        println!("<setup for read byte from addr 4>");

        simulator.set_out_val("ctrl", "out", MemCtrl::Read as SignalUnsigned);
        simulator.set_out_val("size", "out", 1);

        simulator.clock(&mut clock);

        assert_eq!(clock, 3);
        assert_eq!(simulator.get_input_val(out), 0xf0.into());
        assert_eq!(simulator.get_input_val(err), false.into());

        println!("<setup for read byte from addr 4>");
        simulator.set_out_val("size", "out", 1);
        simulator.set_out_val("sign", "out", true);

        simulator.clock(&mut clock);
        assert_eq!(clock, 4);
        assert_eq!(simulator.get_input_val(out), 0xffff_fff0.into());
        assert_eq!(simulator.get_input_val(err), false.into());

        println!("<setup for read half-word from addr 4>");
        simulator.set_out_val("size", "out", 2);
        simulator.set_out_val("sign", "out", true);

        simulator.clock(&mut clock);
        assert_eq!(clock, 5);
        assert_eq!(simulator.get_input_val(out), 0x0000_00f0.into());
        assert_eq!(simulator.get_input_val(err), false.into());

        println!("<setup for read word from addr 4>");
        simulator.set_out_val("size", "out", 4);
        simulator.set_out_val("sign", "out", true);
        simulator.clock(&mut clock);
        assert_eq!(clock, 6);
        assert_eq!(simulator.get_input_val(out), 0x0000_00f0.into());
        assert_eq!(simulator.get_input_val(err), false.into());

        println!("<setup for write half-word at add 10>");
        simulator.set_out_val("addr", "out", 10); // b
        simulator.set_out_val("data", "out", 0x1234);
        simulator.set_out_val("ctrl", "out", MemCtrl::Write as SignalUnsigned);
        simulator.set_out_val("size", "out", 2);

        simulator.clock(&mut clock);
        assert_eq!(clock, 7);
        assert_eq!(simulator.get_input_val(err), false.into());

        println!("<setup for read byte at add 10>");
        simulator.set_out_val("ctrl", "out", MemCtrl::Read as SignalUnsigned);
        simulator.set_out_val("size", "out", 1);
        simulator.clock(&mut clock);
        assert_eq!(clock, 8);
        assert_eq!(simulator.get_input_val(out), 0x34.into());

        println!("<setup for read byte at add 11>");
        simulator.set_out_val("addr", "out", 11);
        simulator.clock(&mut clock);
        assert_eq!(clock, 9);
        assert_eq!(simulator.get_input_val(out), 0x12.into());
    }
}
