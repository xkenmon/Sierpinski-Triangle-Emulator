use std::{fmt::Debug, time::Duration};

use iced::{
    canvas::{Cache, Frame, Geometry, Path, Program, Stroke},
    executor, time, Application, Canvas, Command, Container, Length, Point, Settings, Size,
    Subscription,
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
    fix_points: Vec<Point>,
    random_points: Vec<Point>,
    bound: Size<f32>,
    cache: Cache,
    version: usize,
    duration: std::time::Duration,
}

#[derive(Debug)]
pub enum Message {
    AddRandomPoint,
    ReDraw,
}

#[derive(Debug)]
pub enum TickError {
    CommonError,
}

impl Application for SierpinskiEmulator {
    type Executor = executor::Default;

    type Message = Message;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let bound = Size::new(400_f32, 400_f32);
        let emulator = SierpinskiEmulator {
            fix_points: vec![
                Point::new(200_f32, 400_f32),
                Point::new(0_f32, 0_f32),
                Point::new(400_f32, 0_f32),
            ],
            random_points: vec![],
            cache: Cache::new(),
            bound,
            version: 0,
            duration: Duration::from_millis(50),
        };
        (emulator, Command::none())
    }

    fn title(&self) -> String {
        "Sierpinski Emulator".to_string()
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> iced::Command<Self::Message> {
        match message {
            Message::AddRandomPoint => {
                self.version += 1;
                let p = self.gen_rand_point();
                self.random_points.push(p);
                if self.version % 100 == 0 {
                    self.cache.clear();
                }
            }
            Message::ReDraw => {
                self.cache.clear();
            }
        }

        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let bound = self.bound;
        let canvas = Canvas::new(self)
            .width(Length::Units(bound.width as u16))
            .height(Length::Units(bound.height as u16));

        Container::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(self.duration).map(|_| Message::AddRandomPoint)
    }
}

impl Program<Message> for SierpinskiEmulator {
    fn draw(
        &self,
        bounds: iced::Rectangle,
        _cursor: iced::canvas::Cursor,
    ) -> Vec<iced::canvas::Geometry> {
        let geom = self.cache.draw(bounds.size(), |frame| {
            self.random_points.iter().for_each(|p| {
                let path = Path::rectangle(*p, Size::new(1_f32, 1_f32));
                frame.stroke(&path, Stroke::default())
            });
            self.fix_points.iter().for_each(|p| {
                let path = Path::rectangle(*p, Size::new(2_f32, 2_f32));
                frame.stroke(&path, Stroke::default())
            });
        });

        vec![geom]
    }
}

impl SierpinskiEmulator {
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
