use std::{fmt::Debug, ops::Not};

use iced::{
    canvas::{event, Cache, Event, Path, Program, Stroke},
    executor, slider, Application, Canvas, Color, Column, Command, Length, Point, Row, Settings,
    Size, Slider, Text,
};

use rand::Rng;

fn main() -> iced::Result {
    SierpinskiEmulator::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Debug)]
struct SierpinskiEmulator {
    graph: SierpinskiGraph,
    max_iter_state: slider::State,
    cur_iter_state: slider::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetMaxIter(i32),
    SetCurIter(i32),
    DrawCurIter(i32),
    AddFixPoint(Point),
    RemoveFixPoint,
}

impl Application for SierpinskiEmulator {
    type Executor = executor::Default;

    type Message = Message;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let emulator = SierpinskiEmulator {
            max_iter_state: slider::State::default(),
            cur_iter_state: slider::State::default(),
            graph: SierpinskiGraph::new(),
        };
        (emulator, Command::none())
    }

    fn title(&self) -> String {
        "Sierpinski Triangle Emulator".to_string()
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> iced::Command<Self::Message> {
        match message {
            Message::SetMaxIter(max_iter) => {
                self.graph.max_iter = max_iter;
                while self.graph.random_points.len() < max_iter as usize {
                    let p = self.graph.gen_rand_point();
                    self.graph.random_points.push(p);
                }
            }
            Message::SetCurIter(cur_iter) => {
                if cur_iter > self.graph.max_iter {
                    self.graph.cur_iter = self.graph.max_iter;
                } else {
                    self.graph.cur_iter = cur_iter;
                }
            }
            Message::AddFixPoint(point) => {
                self.graph.fix_points.push(point);
                self.graph.random_points.clear();
                self.graph.max_iter = 0;
                self.graph.cur_iter = 0;
            }
            Message::RemoveFixPoint => {
                self.graph.fix_points.pop();
                self.graph.random_points.clear();
                self.graph.max_iter = 0;
                self.graph.cur_iter = 0;
            }
            Message::DrawCurIter(cur_iter) => {
                self.graph.cur_iter = cur_iter;
            }
        }
        self.graph.redraw();

        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let bound = self.graph.bound;
        let max_iter = self.graph.max_iter;
        let cur_iter = self.graph.cur_iter;
        let fix_point_is_empty = self.graph.fix_points.is_empty();

        let mut content = Column::new()
            .width(Length::Fill)
            .align_items(iced::Align::Center)
            .push(
                Canvas::new(&mut self.graph)
                    .width(Length::Units(bound.width as u16))
                    .height(Length::Units(bound.height as u16)),
            );
        if fix_point_is_empty.not() {
            content = content
                .push(
                    Row::new()
                        .padding(10)
                        .spacing(20)
                        .push(Text::new(format!("max iter: {:?}", max_iter)))
                        .push(
                            Slider::new(
                                &mut self.max_iter_state,
                                0..=10000,
                                max_iter,
                                Message::SetMaxIter,
                            )
                            .width(Length::Units(bound.width as u16)),
                        ),
                )
                .push(
                    Row::new()
                        .padding(10)
                        .spacing(20)
                        .push(Text::new(format!("cur iter: {:?}", cur_iter)))
                        .push(
                            Slider::new(
                                &mut self.cur_iter_state,
                                0..=10000,
                                cur_iter,
                                Message::SetCurIter,
                            )
                            .width(Length::Units(bound.width as u16)),
                        ),
                );
        }
        content.into()
    }
}

#[derive(Debug)]
struct SierpinskiGraph {
    max_iter: i32,
    cur_iter: i32,
    fix_points: Vec<Point>,
    random_points: Vec<Point>,
    bound: Size<f32>,
    cache: Cache,
}

impl Program<Message> for SierpinskiGraph {
    fn update(
        &mut self,
        event: iced::canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::canvas::Cursor,
    ) -> (iced::canvas::event::Status, Option<Message>) {
        let cursor_position = if let Some(position) = cursor.position_in(&bounds) {
            position
        } else {
            return (event::Status::Ignored, None);
        };

        match event {
            Event::Mouse(mouse_event) => {
                let message = match mouse_event {
                    iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                        Some(Message::AddFixPoint(cursor_position))
                    }
                    iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right) => {
                        Some(Message::RemoveFixPoint)
                    }
                    _ => None,
                };
                (event::Status::Captured, message)
            }
            _ => (event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        bounds: iced::Rectangle,
        _cursor: iced::canvas::Cursor,
    ) -> Vec<iced::canvas::Geometry> {
        let geom = self.cache.draw(bounds.size(), |frame| {
            frame.stroke(
                &Path::rectangle(Point::ORIGIN, frame.size()),
                Stroke::default(),
            );
            self.random_points[0..self.cur_iter as usize]
                .iter()
                .for_each(|p| {
                    let path = Path::rectangle(*p, Size::new(1_f32, 1_f32));
                    frame.stroke(&path, Stroke::default())
                });
            self.fix_points.iter().for_each(|p| {
                let path = Path::circle(*p, 5.0);
                frame.fill(&path, Color::from_rgb8(0x12, 0x93, 0xD8));
            });
        });

        vec![geom]
    }
}

impl SierpinskiGraph {
    fn new() -> SierpinskiGraph {
        SierpinskiGraph {
            max_iter: 0,
            cur_iter: 0,
            fix_points: vec![],
            random_points: vec![],
            bound: Size::new(600.0, 600.0),
            cache: Cache::new(),
        }
    }

    fn redraw(&mut self) {
        self.cache.clear();
    }

    fn gen_rand_point(&self) -> Point {
        let dest_point_idx = rand::thread_rng().gen_range(0..self.fix_points.len());
        let dest_point = self.fix_points[dest_point_idx];
        let cur_point = self
            .random_points
            .last()
            .or_else(|| Some(&self.fix_points[0]))
            .unwrap();
        let new_point = Point::new(
            (dest_point.x + cur_point.x) / 2_f32,
            (dest_point.y + cur_point.y) / 2_f32,
        );
        new_point
    }
}
