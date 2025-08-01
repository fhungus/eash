use std::io::Stdout;

pub trait Element {
    fn get_width(&self) -> u32;

    // we're just gonna use stdout until i can grow a brain
    fn render(&self, start_position: i32, w: &mut Stdout);
}
