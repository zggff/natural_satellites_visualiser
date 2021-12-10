use std::time::Duration;

use druid::kurbo::{Line, Rect};
use druid::piet::kurbo::Shape;
use druid::piet::Color;
use druid::piet::TextLayoutBuilder;
use druid::RenderContext;
use druid::{
    kurbo::Ellipse, piet::Text, widget::ListIter, BoxConstraints, Data, Env, Event, EventCtx,
    LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx, Point, Size, TimerToken, UpdateCtx, Widget,
};
use evalexpr::*;
use satellite_data::{
    database::Database,
    satellites::{MajorBody, Satellite},
};

#[derive(Clone, Debug)]
pub struct SatelliteteVec(pub Vec<SatelliteWrapper>);

impl Data for SatelliteteVec {
    fn same(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        if !self.0.is_empty() {
            for (i, sat) in self.0.iter().enumerate() {
                if sat != other.0.get(i).unwrap() {
                    return false;
                }
            }
        }
        true
    }
}

impl ListIter<SatelliteWrapper> for SatelliteteVec {
    fn for_each(&self, mut cb: impl FnMut(&SatelliteWrapper, usize)) {
        for satellite in self.0.iter() {
            cb(satellite, self.0.len());
        }
        // cb (self.0.get(0).unwrap(), 1)
    }
    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut SatelliteWrapper, usize)) {
        for satellite in self.0.iter_mut() {
            cb(satellite, (*satellite).satellite)
        }
    }
    fn data_len(&self) -> usize {
        self.0.len()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct SatelliteWrapper {
    pub satellite: usize,
    pub selected: bool,
}

impl Copy for SatelliteWrapper {}

impl Data for SatelliteWrapper {
    fn same(&self, other: &Self) -> bool {
        self.satellite == other.satellite && self.selected == other.selected
    }
}
pub struct AstronomyCanvas {
    pub selected_satellites: Vec<Satellite>,
    pub selected_satellite: Option<SatelliteWrapper>,
    pub move_bul: bool,
    pub center: Point,
    pub database: Database,
    pub full_database: Database,
    pub count_change_timer: Option<TimerToken>,
    pub selected_update_timer: Option<TimerToken>,
}
#[derive(Clone, Data, Lens, Debug)]
pub struct AstronomyCanvasData {
    pub all_displayed: usize,
    pub selected: bool,
    pub scale: f64,
    pub center: Point,
    pub toggle_distance: bool,
    pub toggle_angle: bool,
    pub toggle_major_semiaxes: bool,
    pub mouse_point: Option<Point>,
    pub selected_satellites: SatelliteteVec,
    pub match_string: String,
    pub selected_satellite: Option<SatelliteWrapper>,
    pub graph_view: bool,
    pub x_value: String,
    pub y_value: String,
}

impl Widget<AstronomyCanvasData> for AstronomyCanvas {
    fn paint(&mut self, ctx: &mut PaintCtx, data: &AstronomyCanvasData, env: &Env) {
        if !data.graph_view {
            self.render_circular_view(ctx, data, env);
        } else {
            self.render_graph_view(ctx, data, env);
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AstronomyCanvasData,
        _env: &Env,
    ) -> Size {
        let default_size = Size::new(
            ctx.window().get_size().width,
            ctx.window().get_size().height,
        );

        bc.constrain(default_size)
    }

    fn event(
        &mut self,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AstronomyCanvasData,
        _env: &Env,
    ) {
        match event {
            Event::Wheel(mouse_event) => {
                data.mouse_point = None;

                let (_, y) = mouse_event.wheel_delta.into();

                if y.is_sign_positive() {
                    data.scale *= 2.0
                } else {
                    data.scale /= 2.0
                }
                ctx.request_paint();
                ctx.request_layout();
            }
            Event::MouseDown(mouse_event) => {
                match mouse_event.button {
                    druid::MouseButton::Left => {
                        data.mouse_point = None;

                        let mouse_pos = mouse_event.pos;
                        data.center = mouse_pos;
                        ctx.set_active(true);
                    }
                    druid::MouseButton::Right => {
                        let mouse_pos = mouse_event.pos;
                        data.mouse_point = Some(mouse_pos);
                        self.selected_satellites.clear();
                    }
                    druid::MouseButton::Middle => {
                        data.selected = !data.selected
                        // data.mouse_point = None;
                    }
                    _ => {}
                }

                ctx.request_paint();
                ctx.request_layout();
            }
            Event::MouseMove(mouse_event) => {
                if ctx.is_active() {
                    self.move_bul = true;
                    let mouse_pos = mouse_event.pos;
                    data.center = mouse_pos;
                }
            }
            Event::MouseUp(_mouse_event) => {
                ctx.set_active(false);
                let satellites: Vec<SatelliteWrapper> = self
                    .selected_satellites
                    .clone()
                    .into_iter()
                    .map(|satellite| SatelliteWrapper {
                        satellite: satellite.id,
                        selected: false,
                    })
                    .collect();
                data.selected_satellites = SatelliteteVec(satellites);
                self.move_bul = false;
            }
            Event::Timer(_event) => {
                ctx.request_layout();
                ctx.request_paint();
                data.all_displayed = self.database.data.len();
            }
            _ => {}
        };
        data.selected_satellite = self.selected_satellite;
    }
    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &AstronomyCanvasData,
        data: &AstronomyCanvasData,
        _env: &Env,
    ) {
        if old_data.selected_satellites.0.len() == data.selected_satellites.0.len()
            && !data.selected_satellites.0.is_empty()
        {
            let satellite: Vec<SatelliteWrapper> = data
                .selected_satellites
                .0
                .clone()
                .iter()
                .enumerate()
                .filter(|(id, satellite)| {
                    let old = old_data.selected_satellites.0.get(*id).unwrap();
                    satellite.satellite == old.satellite && satellite.selected != old.selected
                })
                .map(|(_id, satellite)| *satellite)
                .collect();
            if !satellite.is_empty() {
                self.selected_satellite = Some(*satellite.first().unwrap());
                ctx.request_timer(Duration::from_millis(1));
            }
        }
        if data.toggle_distance != old_data.toggle_distance
            || data.toggle_angle != old_data.toggle_angle
            || data.toggle_major_semiaxes != old_data.toggle_major_semiaxes
        {
            ctx.request_paint();
            ctx.request_layout();
        }
        if old_data.center != data.center {
            if self.move_bul {
                self.center = Point::new(
                    self.center.x + old_data.center.x - data.center.x,
                    self.center.y + old_data.center.y - data.center.y,
                );
            }
            ctx.request_paint();
            ctx.request_layout();
        }
        if data.match_string != old_data.match_string {
            if data.match_string.is_empty() {
                self.database = self.full_database.clone();
                return;
            }
            let database = &self.full_database;
            let database: Vec<Satellite> = database
                .data
                .iter()
                .cloned()
                .filter(|satellite: &Satellite| {
                    parse_logicall_expression(satellite, &data.match_string)
                })
                .collect();
            let database = Database { data: database };
            self.database = database;
            // let all_displayed = self.database.data.len();
            // self.all_displayed = all_displayed
            ctx.request_paint();
            ctx.request_layout();
            ctx.request_timer(Duration::from_millis(1));
        }
    }
    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &AstronomyCanvasData,
        _env: &Env,
    ) {
    }
}

impl AstronomyCanvas {
    fn render_circular_view(&mut self, ctx: &mut PaintCtx, data: &AstronomyCanvasData, _env: &Env) {
        let scale = data.scale * 5000.0;
        for satellite in &self.database.data {
            let a: f64 = satellite.orbital_params.major_semiaxis;
            let e: f64 = satellite.orbital_params.eccentricity;
            let b = (a.powi(2) * (1.0 - e.powi(2))).sqrt();
            let color = match satellite.major_body {
                MajorBody::Earth => Color::rgb(0.0, 1.0, 0.4),
                MajorBody::Mars => Color::rgb(1.0, 0.0, 0.0),
                MajorBody::Jupiter => Color::rgb(1.0, 0.5, 0.3),
                MajorBody::Saturn => Color::rgb(1.0, 0.9, 0.3),
                MajorBody::Uranus => Color::rgb(0.3, 1.0, 0.8),
                MajorBody::Neptune => Color::rgb(0.1, 0.2, 1.0),
                MajorBody::Pluto => Color::rgb(0.9, 0.2, 1.0),
            };

            let rotation: f64 = satellite.orbital_params.inclination;
            let rotation: f64 = rotation.to_radians();

            let x = self.center.x + ((a * e) * rotation.cos() / scale);
            let y = self.center.y + ((a * e) * rotation.sin() / scale);
            let ellipse = Ellipse::new((x, y), (a / scale, b / scale), rotation);

            match data.mouse_point {
                Some(mouse_point) => {
                    let x = mouse_point.x;
                    let y = mouse_point.y;
                    let rect = Rect::from_center_size((x, y), (40.0, 40.0));
                    let diagonal = Line::new((rect.x0, rect.y0), (rect.x1, rect.y1));
                    // let diagonal2 = Line::new((rect.x0, rect.y0), (rect.x1, rect.y1));
                    ctx.stroke(rect, &Color::AQUA, 1.0);
                    ctx.stroke(diagonal, &Color::AQUA, 1.0);

                    let ell = ellipse.to_path(0.01);
                    let ell = ell.segments();
                    for ell in ell {
                        if !ell.intersect_line(diagonal).is_empty()
                            && !self.selected_satellites.contains(satellite)
                        {
                            self.selected_satellites.push(satellite.clone());
                        }
                    }
                }
                None => {
                    self.selected_satellites.clear();
                }
            }

            if data.toggle_major_semiaxes {
                let x1 = x + (a * rotation.cos() / scale);
                let y1 = y + (a * rotation.sin() / scale);
                let rotation: f64 = satellite.orbital_params.ascending_node;
                let rotation: f64 = (rotation + 180.0).to_radians();
                let x2 = x + (a * rotation.cos() / scale);
                let y2 = y + (a * rotation.sin() / scale);
                let major_semiaxes = Line::new((x2, y2), (x1, y1));

                let rotation: f64 = satellite.orbital_params.ascending_node;
                let rotation: f64 = (rotation + 90.0).to_radians();
                let x1 = x + (b * rotation.cos() / scale);
                let y1 = y + (b * rotation.sin() / scale);
                let rotation: f64 = satellite.orbital_params.ascending_node;
                let rotation: f64 = (rotation + 270.0).to_radians();
                let x2 = x + (b * rotation.cos() / scale);
                let y2 = y + (b * rotation.sin() / scale);
                let minor_semiaxes = Line::new((x2, y2), (x1, y1));

                ctx.stroke(major_semiaxes, &color.clone().with_alpha(0.5), 1.0);
                ctx.stroke(minor_semiaxes, &color.clone().with_alpha(0.5), 1.0);
            }

            ctx.stroke(ellipse, &color, 1.0);
        }
        let planet = Ellipse::new(self.center, (5., 5.), 0.0);
        ctx.fill(planet, &Color::rgb(1.0, 1.0, 0.0));

        if data.toggle_distance {
            let line = Line::new(self.center, (ctx.size().width, self.center.y));
            ctx.stroke(line, &Color::WHITE, 2.0);
            for i in 0..50 {
                let distance = i as f64 * 80.0;
                let line = Line::new(
                    (self.center.x + distance, self.center.y - 15.0),
                    (self.center.x + distance, self.center.y + 15.0),
                );
                let text = ctx.text();
                let text = text
                    .new_text_layout((distance * scale).to_string())
                    .text_color(Color::WHITE);
                let text = text.build().unwrap();
                ctx.draw_text(&text, (self.center.x + distance, self.center.y + 20.0));
                ctx.stroke(line, &Color::WHITE, 1.0);
            }
        }
        if data.toggle_angle {
            let ellipse = Ellipse::new(self.center, (150.0, 150.0), 0.0);
            ctx.stroke(ellipse, &Color::WHITE, 2.0);
            for i in 0..36 {
                let rotation: f64 = (i * 10).into();
                let rotation: f64 = rotation.to_radians();
                let distance = 150.0;

                let x1 = self.center.x + ((distance - 5.0) * rotation.cos());
                let y1 = self.center.y + ((distance - 5.0) * rotation.sin());

                let x2 = self.center.x + ((distance + 5.0) * rotation.cos());
                let y2 = self.center.y + ((distance + 5.0) * rotation.sin());

                let line = Line::new((x1, y1), (x2, y2));
                ctx.stroke(line, &Color::WHITE, 1.0);

                let x = self.center.x + ((distance + 30.0) * rotation.cos()) - 10.0;
                let y = self.center.y + ((distance + 30.0) * rotation.sin());

                let text = ctx.text();
                let text = text
                    .new_text_layout((i * 10).to_string())
                    .text_color(Color::WHITE);
                let text = text.build().unwrap();
                ctx.draw_text(&text, (x, y));
            }
        }
    }
    fn render_graph_view(&mut self, ctx: &mut PaintCtx, data: &AstronomyCanvasData, _env: &Env) {
        let x_values: Vec<&str> = data
            .x_value
            .split('|')
            .filter(|&value| !value.is_empty() && value != " ")
            .collect();

        let mut x_scale = 1.0;
        let mut x_value = String::new();
        if !x_values.is_empty() {
            x_value = x_values[0].to_string();
        }
        if x_values.len() > 1 {
            let value = x_values[1];
            let value = eval(value);
            let value: f64 = match value {
                Ok(value) => value.as_number().unwrap_or(1.0),
                Err(_) => 1.0,
            };
            x_scale = value;
        }
        x_scale *= data.scale;

        let y_values: Vec<&str> = data
            .y_value
            .split('|')
            .filter(|&value| !value.is_empty() && value != " ")
            .collect();
        let mut y_value = String::new();

        let mut y_scale = 1.0;
        if !y_values.is_empty() {
            y_value = y_values[0].to_string();
        }
        if y_values.len() > 1 {
            let value = y_values[1];
            let value = eval(value);
            let value: f64 = match value {
                Ok(value) => value.as_number().unwrap_or(1.0),
                Err(_) => 1.0,
            };
            y_scale = value;
        }
        y_scale *= data.scale;

        let x_line = Line::new(
            (self.center.x - 1000.0, self.center.y),
            (self.center.x + 1000.0, self.center.y),
        );
        let y_line = Line::new(
            (self.center.x, self.center.y - 1000.0),
            (self.center.x, self.center.y + 1000.0),
        );
        ctx.stroke(x_line, &Color::WHITE, 2.0);
        ctx.stroke(y_line, &Color::WHITE, 2.0);
        for i in -50..50 {
            let distance = i as f64 * 80.0;
            let x_line = Line::new(
                (self.center.x + distance, self.center.y + 10.0),
                (self.center.x + distance, self.center.y - 10.0),
            );
            let y_line = Line::new(
                (self.center.x + 10.0, self.center.y + distance),
                (self.center.x - 10.0, self.center.y + distance),
            );
            ctx.stroke(x_line, &Color::WHITE, 1.0);
            ctx.stroke(y_line, &Color::WHITE, 1.0);
            let text = ctx.text();
            let text_x = text
                .new_text_layout((distance * x_scale).to_string())
                .text_color(Color::WHITE);
            let text_x = text_x.build().unwrap();
            let text_y = text
                .new_text_layout((distance * y_scale).to_string())
                .text_color(Color::WHITE);
            let text_y = text_y.build().unwrap();

            ctx.draw_text(&text_y, (self.center.x - 20.0, self.center.y + distance));
            ctx.draw_text(&text_x, (self.center.x + distance, self.center.y + 20.0));
        }

        for satellite in &self.database.data {
            let x = parse_math_expression(satellite, &x_value);
            let y = parse_math_expression(satellite, &y_value);

            let x = x / x_scale;
            let y = y / y_scale;

            let x = self.center.x + x;
            let y = self.center.y - y;

            let ellipse = Ellipse::new((x, y), (3.0, 3.0), 0.0);

            match data.mouse_point {
                Some(mouse_point) => {
                    let rect = Rect::from_center_size((mouse_point.x, mouse_point.y), (20.0, 20.0));
                    if x < rect.x1
                        && x > rect.x0
                        && y < rect.y1
                        && y > rect.y0
                        && !self.selected_satellites.contains(satellite)
                    {
                        self.selected_satellites.push(satellite.clone());
                    }
                    ctx.stroke(rect, &Color::AQUA, 1.0);
                }
                None => {
                    self.selected_satellites.clear();
                }
            }

            let color = match satellite.major_body {
                MajorBody::Earth => Color::rgb(0.0, 1.0, 0.4),
                MajorBody::Mars => Color::rgb(1.0, 0.0, 0.0),
                MajorBody::Jupiter => Color::rgb(1.0, 0.5, 0.3),
                MajorBody::Saturn => Color::rgb(1.0, 0.9, 0.3),
                MajorBody::Uranus => Color::rgb(0.3, 1.0, 0.8),
                MajorBody::Neptune => Color::rgb(0.1, 0.2, 1.0),
                MajorBody::Pluto => Color::rgb(0.9, 0.2, 1.0),
            };
            ctx.fill(ellipse, &color);
        }
    }
}

fn parse_logicall_expression(satellite: &Satellite, logical_expression: &str) -> bool {
    let precompiled = build_operator_tree(logical_expression);
    let precompiled = match precompiled {
        Ok(precompiled) => precompiled,
        Err(_e) => build_operator_tree("false").unwrap(),
    };
    let context = context_map! {

        "a" => satellite.orbital_params.major_semiaxis,
        "i" => satellite.orbital_params.inclination,
        "e" => satellite.orbital_params.eccentricity,
        "node" => satellite.orbital_params.ascending_node,
        "gm" => satellite.physical_params.gm.to_value(),
        "radius" => satellite.physical_params.radius.to_value(),
        "density" => satellite.physical_params.density.to_value(),
        "magnitude" => satellite.physical_params.magnitude.to_value(),
        "albedo" => satellite.physical_params.albedo.to_value(),
        "mb" => satellite.major_body.to_string(),
        "name" => satellite.name.clone()
    }
    .unwrap();
    let result = precompiled.eval_boolean_with_context(&context);
    match result {
        Ok(result) => result,
        _ => false,
    }
}

fn parse_math_expression(satellite: &Satellite, math_expression: &str) -> f64 {
    let precompiled = build_operator_tree(math_expression);
    let precompiled = match precompiled {
        Ok(precompiled) => precompiled,
        Err(_e) => build_operator_tree("false").unwrap(),
    };
    let context = context_map! {
        "a" => satellite.orbital_params.major_semiaxis,
        "i" => satellite.orbital_params.inclination,
        "e" => satellite.orbital_params.eccentricity,
        "node" => satellite.orbital_params.ascending_node,
        "gm" => satellite.physical_params.gm.to_value(),
        "radius" => satellite.physical_params.radius.to_value(),
        "density" => satellite.physical_params.density.to_value(),
        "magnitude" => satellite.physical_params.magnitude.to_value(),
        "albedo" => satellite.physical_params.albedo.to_value(),
    }
    .unwrap();

    let result = precompiled.eval_float_with_context(&context);
    match result {
        Ok(result) => result,
        _ => 0.0,
    }
}
