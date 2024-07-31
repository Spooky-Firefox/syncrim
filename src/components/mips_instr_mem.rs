// use std::fmt::Alignment;
#[cfg(feature = "gui-egui")]
use crate::common::EguiComponent;
use crate::common::{
    Component, Condition, Id, Input, InputPort, OutputType, Ports, SignalSigned, SignalUnsigned,
    SignalValue, Simulator,
};
use log::*;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::rc::Rc;

pub const INSTR_MEM_A_IN_ID: &str = "instr_mem_a_in";

pub const INSTR_MEM_RD_OUT_ID: &str = "rd_out";

#[derive(Serialize, Deserialize, Clone)]
pub struct InstrMem {
    pub(crate) id: Id,
    pub(crate) pos: (f32, f32),
    pub(crate) a_in: Input,

    #[serde(skip)]
    pub memory: Memory,
    history: RefCell<Vec<MemOp>>,
    #[serde(skip)]
    pub init_state: BTreeMap<usize, u8>,
}

#[derive(Serialize, Deserialize, Clone)]
struct MemOp {
    pub data: Option<usize>,
    pub addr: usize,
    pub size: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Memory(pub Rc<RefCell<BTreeMap<usize, u8>>>);

impl Default for Memory {
    fn default() -> Self {
        Self::new(BTreeMap::new())
    }
}

impl Memory {
    pub fn new(data: BTreeMap<usize, u8>) -> Self {
        Memory(Rc::new(RefCell::new(data)))
    }

    fn align(&self, addr: usize, size: usize) -> SignalValue {
        ((addr % size != 0) as SignalUnsigned).into()
    }

    pub fn read(&self, addr: usize, size: usize, sign: bool, big_endian: bool) -> SignalValue {
        let data: Vec<u8> = (0..size)
            .map(|i| *self.0.borrow().get(&(addr + i)).unwrap_or(&0))
            .collect();

        let data = data.as_slice();

        //trace!("{:x?}", data);

        match size {
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
        }
        .into()
    }

    pub fn write(&self, addr: usize, size: usize, big_endian: bool, data: SignalValue) {
        let data: SignalUnsigned = data.try_into().unwrap();
        trace!("we = 1, now writing {:?} at addr {:?}", data, addr);

        match size {
            1 => {
                trace!("write byte");
                self.0.borrow_mut().insert(addr, data as u8);
            }
            2 => {
                if big_endian {
                    trace!("write half word be");
                    (data as u16)
                        .to_be_bytes()
                        .iter()
                        .enumerate()
                        .for_each(|(i, bytes)| {
                            self.0.borrow_mut().insert(addr + i, *bytes);
                        })
                } else {
                    trace!("write half word le");
                    (data as u16)
                        .to_le_bytes()
                        .iter()
                        .enumerate()
                        .for_each(|(i, bytes)| {
                            self.0.borrow_mut().insert(addr + i, *bytes);
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
                            self.0.borrow_mut().insert(addr + i, *bytes);
                        })
                } else {
                    trace!("write word le");
                    data.to_le_bytes()
                        .iter()
                        .enumerate()
                        .for_each(|(i, bytes)| {
                            self.0.borrow_mut().insert(addr + i, *bytes);
                        })
                }
            }
            _ => {
                panic!("illegal sized memory operation, size = {}", size)
            }
        };
    }
}

#[typetag::serde]
impl Component for InstrMem {
    fn to_(&self) {
        trace!("instr_mem");
    }
    #[cfg(feature = "gui-egui")]

    fn get_id_ports(&self) -> (Id, Ports) {
        (
            self.id.clone(),
            Ports::new(
                vec![&InputPort {
                    port_id: INSTR_MEM_A_IN_ID.to_string(),
                    input: self.a_in.clone(),
                }],
                OutputType::Combinatorial,
                vec![INSTR_MEM_RD_OUT_ID],
            ),
        )
    }

    fn set_id_port(&mut self, target_port_id: Id, new_input: Input) {
        match target_port_id.as_str() {
            INSTR_MEM_A_IN_ID => self.a_in = new_input,
            _ => {}
        }
    }

    // propagate sign extension to output
    // TODO: always extend to Signal size? (it should not matter and should be slightly cheaper)
    fn clock(&self, simulator: &mut Simulator) -> Result<(), Condition> {
        let a1_addr: u32 = simulator.get_input_value(&self.a_in).try_into().unwrap();

        let big_endian: bool = true;
        let sign: bool = false;
        let size: u32 = 4;

        // trace!("read addr {:?} size {:?}", a1_addr, size);
        let value1 = self
            .memory
            .read(a1_addr as usize, size as usize, sign, big_endian)
            .try_into()
            .unwrap();

        simulator.set_out_value(&self.id, INSTR_MEM_RD_OUT_ID, SignalValue::Data(value1));

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Deref for Memory {
    type Target = RefCell<BTreeMap<usize, u8>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl InstrMem {
    pub fn new(id: &str, pos: (f32, f32), a_in: Input, memory: BTreeMap<usize, u8>) -> Self {
        InstrMem {
            id: id.to_string(),
            pos,
            a_in,
            memory: Memory::new(memory.clone()),
            history: RefCell::new(vec![]),
            init_state: memory.clone(),
        }
    }

    pub fn rc_new(id: &str, pos: (f32, f32), a_in: Input) -> Rc<Self> {
        let mut mem = BTreeMap::new();
        //fill the defined memory range with zeroes
        for i in 0..1.clone() {
            mem.insert(i as usize, 0u8);
        }
        Rc::new(InstrMem::new(id, pos, a_in, mem))
    }
}
