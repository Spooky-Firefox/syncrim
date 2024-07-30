use std::path::PathBuf;
use std::rc::Rc;
use syncrim::common::EguiComponent;
#[cfg(feature = "gui-egui")]
use syncrim::gui_egui::editor::Library;
use syncrim::{
    common::{ComponentStore, Input},
    components::*,
    fern::fern_setup,
};

fn main() {
    fern_setup();
    let cs = ComponentStore {
        store: vec![
            DataMem::rc_new(
                "data_mem",
                (200.0, 200.0),
                Input::new("a", "out"),
                Input::new("op", "out"),
                Input::new("wd", "out"),
                Input::new("we", "out"),
                true,
            ),
            ProbeEdit::rc_new("a", (60.0, 100.0)),
            ProbeEdit::rc_new("op", (60.0, 120.0)),
            ProbeEdit::rc_new("wd", (60.0, 140.0)),
            ProbeEdit::rc_new("we", (60.0, 160.0)),
            Probe::rc_new(
                "p1",
                (270.0, 120.0),
                Input::new("data_mem", DATA_MEM_RD_OUT_ID),
            ),
        ],
    };

    let cs = autowire(cs);

    let path = PathBuf::from("add.json");
    cs.save_file(&path);

    #[cfg(feature = "gui-egui")]
    syncrim::gui_egui::gui(cs, &path, Library::default()).ok();

    #[cfg(feature = "gui-vizia")]
    syncrim::gui_vizia::gui(cs, &path);
}

fn autowire(mut cs: ComponentStore) -> ComponentStore {
    let mut x = 1;
    let mut tmp_vec: Vec<Rc<dyn EguiComponent>> = vec![];

    for c in &cs.store {
        let (id, ports) = c.get_id_ports();
        println!("{}", id);

        let number_of_inputs = ports.inputs.len();

        for n in 0..number_of_inputs {
            println!("{:?}", ports.inputs[n].input.id);
            println!("{:?}", ports);

            let id_tmp = format!("{id}_w{n}");
            //let pos_temp = vec![];
            let input = ports.inputs[n].input.clone();

            let starting_pos = c.get_pos();

            let mut destination_pos = (starting_pos.0 - 50.0, starting_pos.1);

            let default_input = Input::new(&format!("c{n}"), "out");
            let mut w = Wire::rc_new(&id_tmp, vec![starting_pos, destination_pos], default_input);

            for d in &cs.store {
                let (id2, ports2) = d.get_id_ports();
                let input = Input::new(&id2, "out");
                if id2 == ports.inputs[n].input.id {
                    destination_pos = d.get_pos();
                    w = Wire::rc_new(&id_tmp, vec![starting_pos, destination_pos], input.clone());
                }
            }
            tmp_vec.push(w);

            //TODO: get pos and set to destination, look through list of components a second time, then add that as starting pos.

            //println!("{}", id_tmp);
            //let w = Wire::rc_new("w{}", pos, input)
        }

        //cs_copy.store.push("wow");

        // Wire::rc_new(
        //     "w4",
        //     vec![(220.0, 120.0), (260.0, 120.0)],
        //     Input::new("full_adder", FULL_ADD_OUT_ID),
        // ),

        // Ports {
        //     inputs: [
        //         InputPort {
        //             port_id: "full_add_a_in",
        //             input: Input {
        //                 id: "c1",
        //                 field: "out",
        //             },
        //         },
        //         InputPort {
        //             port_id: "full_add_b_in",
        //             input: Input {
        //                 id: "c2",
        //                 field: "out",
        //             },
        //         },
        //         InputPort {
        //             port_id: "full_add_op_in",
        //             input: Input {
        //                 id: "c3",
        //                 field: "out",
        //             },
        //         },
        //     ],

        //     out_type: Combinatorial,
        //     outputs: ["out"],
        // }
    }
    cs.store.append(&mut tmp_vec);

    return cs;
}
