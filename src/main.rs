use std::{fs::File};
use satellite_data::{database::Database};
use druid::{self, AppLauncher, Data, Env, Lens, Point, Widget, WidgetExt, WindowDesc, widget::{Button, Checkbox, ClipBox, Container, Either, Flex, Label, List, Scroll, SizedBox, TextBox}};

pub mod astronomy_canvas;

use astronomy_canvas::{AstronomyCanvas, AstronomyCanvasData, SatelliteWrapper, SatelliteteVec};
// use std::sync::Arc

#[macro_use]
extern crate lazy_static;
#[derive(Clone, Data, Lens)]
struct WindowState {
    
}

lazy_static! {
    static ref DATABASE: Database = {  
        let mut json_file = File::open("satellites.json").unwrap();
        let mut orbital = File::open("orbits.txt").unwrap();
        let mut physical = File::open("physical.txt").unwrap();
        // let database = Database::from_json(&mut json_file);
        let database = Database::parse_data(&mut physical, &mut orbital);
        database
    };
}

fn main() {
    let window = WindowDesc::new(build_root_widget)
        .title("zggff")
        .window_size((1400.0, 600.0));
    
    let initaial_state = AstronomyCanvasData {
        all_displayed: DATABASE.data.len(),
        selected: false,
        center: Point::new(300.0, 300.0),
        mouse_point: None,
        selected_satellites: SatelliteteVec(Vec::new()),
        scale: 1.0,
        toggle_angle: false,
        toggle_distance: false,
        toggle_major_semiaxes: false,
        match_string: String::new(),
        selected_satellite: None,
        graph_view: false,
        x_value: String::new(),
        y_value: String::new(),
    };

    AppLauncher::with_window(window)
        .launch(initaial_state)
        .expect("msg")
}

fn build_root_widget() -> impl Widget <AstronomyCanvasData> {
    
    
    let canvas = AstronomyCanvas {
        count_change_timer: None,
        selected_update_timer: None,
        full_database: DATABASE.clone(),
        selected_satellites: Vec::new(),
        selected_satellite: None,
        move_bul: false,
        database: DATABASE.clone(),
        center: Point::new(300.0, 300.0),
    };
    let textfield = TextBox::new()
        .with_placeholder("mb==\"Uranus\"")
        .lens(AstronomyCanvasData::match_string);
    let x_value = TextBox::new()
        .with_placeholder("x = a")
        .lens(AstronomyCanvasData::x_value);
    let y_value = TextBox::new()
        .with_placeholder("y=a")
        .lens(AstronomyCanvasData::y_value);
    let distance_checkbox = Checkbox::new("Toggle Distances").lens(AstronomyCanvasData::toggle_distance);
    let angle_checkbox = Checkbox::new("Toggle Angles").lens(AstronomyCanvasData::toggle_angle);
    let semiaxes_checkbox = Checkbox::new("Toggle Semiaxes").lens(AstronomyCanvasData::toggle_major_semiaxes);
    let graph_checkbox = Checkbox::new("Toggle graph mode").lens(AstronomyCanvasData::graph_view);
    let label = Label::new(|data: &AstronomyCanvasData, _env: &Env| data.all_displayed.to_string());
    let label1 = Label::new(|data: &AstronomyCanvasData, _env: &Env| data.selected_satellites.0.len().to_string());

    let list = List::new(|| {
        Button::new(|data: &SatelliteWrapper, _env: &Env| {
            let sat = DATABASE.get_satellite_by_id(data.satellite);
            match sat {
                Some(sat) => sat.name,
                None => "".to_string()
            }
        }).on_click(|_ctx, data, _env| {
            data.selected = !data.selected;
            // dbg!(data.clone());
        })
    }).lens(AstronomyCanvasData::selected_satellites);

    let scroll = SizedBox::new(Scroll::new(list).vertical()).height(250.0).width(200.0);

    let either = Either::new(
        |data: &AstronomyCanvasData, _env: &Env| data.graph_view, 
        Flex::column()
            .with_child(x_value)
            .with_child(y_value),
        Flex::column()
            .with_child(distance_checkbox)
            .with_child(semiaxes_checkbox)
            .with_child(angle_checkbox));


    let left_controls = SizedBox::new( Container::new( Flex::column()
        .with_child(
            SizedBox::new( textfield)
                .width(200.0)
                .height(200.0)
        )
        .with_child(graph_checkbox)
        .with_child(either)
        .with_child(label)
        .with_child(label1)
        .with_child(scroll)
    )).width(210.0);


    let label = Label::new(|data: &AstronomyCanvasData, _env: &Env| {
        let c = match data.selected_satellite {
            Some(value) => {
                let sat = DATABASE.get_satellite_by_id(value.satellite);
                match sat {
                    Some(sat) => sat.name,
                    None => "".to_string()
                }
            }
            None => "".to_string()
        };
        c
    }).with_text_size(20.0);

    let orbital_data = Label::new(|data: &AstronomyCanvasData, _env: &Env| {
        let c = match data.selected_satellite {
            Some(value) => {
                let sat = DATABASE.get_satellite_by_id(value.satellite);
                match sat {
                    Some(sat) => {
                        let mut satellite_orbital_data = String::new();
                        satellite_orbital_data.push_str("MajorBody:\t");
                        satellite_orbital_data.push_str(&sat.major_body.to_string());
                        satellite_orbital_data.push_str("\n");

                        satellite_orbital_data.push_str("a:\t");
                        satellite_orbital_data.push_str(&sat.orbital_params.major_semiaxis.to_string());
                        satellite_orbital_data.push_str("\n");
                        
                        satellite_orbital_data.push_str("e:\t");
                        satellite_orbital_data.push_str(&sat.orbital_params.eccentricity.to_string());
                        satellite_orbital_data.push_str("\n");

                        satellite_orbital_data.push_str("i:\t");
                        satellite_orbital_data.push_str(&sat.orbital_params.inclination.to_string());
                        satellite_orbital_data.push_str("\n");

                        satellite_orbital_data.push_str("node:\t");
                        satellite_orbital_data.push_str(&sat.orbital_params.ascending_node.to_string());
                        satellite_orbital_data.push_str("\n");


                        satellite_orbital_data
                    },
                    None => "".to_string()
                }
            }
            None => "".to_string()
        };
        c
    });

    let physical_data = Label::new(|data: &AstronomyCanvasData, _env: &Env| {
        let c = match data.selected_satellite {
            Some(value) => {
                let sat = DATABASE.get_satellite_by_id(value.satellite);
                match sat {
                    Some(sat) => {
                        let mut satellite_orbital_data = String::new();
                        satellite_orbital_data.push_str("Gm:\t");
                        satellite_orbital_data.push_str(&sat.physical_params.gm.to_string());
                        satellite_orbital_data.push_str(" km3/sec2\n");

                        satellite_orbital_data.push_str("radius:\t");
                        satellite_orbital_data.push_str(&sat.physical_params.radius.to_string());
                        satellite_orbital_data.push_str(" km\n");
                        
                        satellite_orbital_data.push_str("density:\t");
                        satellite_orbital_data.push_str(&sat.physical_params.density.to_string());
                        satellite_orbital_data.push_str(" g/cm3\n");

                        satellite_orbital_data.push_str("magnitude:\t");
                        satellite_orbital_data.push_str(&sat.physical_params.magnitude.to_string());
                        satellite_orbital_data.push_str("\n");

                        satellite_orbital_data.push_str("albedo:\t");
                        satellite_orbital_data.push_str(&sat.physical_params.albedo.to_string());
                        satellite_orbital_data.push_str("\n");


                        satellite_orbital_data
                    },
                    None => "".to_string()
                }
            }
            None => "".to_string()
        };
        c
    });

    let right_controls = SizedBox::new( 
    Scroll::new(
            Flex::column()
                .with_child(label)
                // .with_child()
                .with_child(orbital_data)
                .with_child(physical_data)
            ).horizontal()
        )
    .width(260.0);

    let canvas = ClipBox::new(
        canvas
    );
    let layout = Flex::row()
        .with_flex_child(left_controls, 0.0)
        .with_flex_child (canvas, 1.0)
        .with_flex_child(right_controls, 0.0);

    layout
        
}