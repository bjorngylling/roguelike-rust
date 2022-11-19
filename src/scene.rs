use ggez::{input::keyboard::KeyInput, Context, GameResult};

pub trait Scene<T> {
    fn update(&mut self, ctx: &mut Context, state: &mut T) -> Transition<T>;
    fn draw(&mut self, ctx: &mut Context, state: &mut T) -> GameResult;
    fn key_down(&mut self, input: KeyInput, repeat: bool) -> Transition<T>;
    fn draw_previous(&self) -> bool {
        false
    }
}

pub enum Transition<T> {
    None,
    Push(Box<dyn Scene<T>>),
    Pop,
    Replace(Box<dyn Scene<T>>),
}

pub struct SceneStack<T> {
    scenes: Vec<Box<dyn Scene<T>>>,
}

impl<T> SceneStack<T> {
    pub fn new(scene: Box<dyn Scene<T>>) -> Self {
        Self {
            scenes: vec![scene],
        }
    }

    pub fn push(&mut self, scene: Box<dyn Scene<T>>) {
        self.scenes.push(scene);
    }

    pub fn pop(&mut self) -> Box<dyn Scene<T>> {
        self.scenes.pop().expect("popped empty scene stack")
    }

    fn switch(&mut self, t: Transition<T>) -> Option<Box<dyn Scene<T>>> {
        match t {
            Transition::None => None,
            Transition::Pop => Some(self.pop()),
            Transition::Push(s) => {
                self.push(s);
                None
            }
            Transition::Replace(s) => {
                let old = self.pop();
                self.push(s);
                Some(old)
            }
        }
    }

    pub fn update(&mut self, ctx: &mut Context, state: &mut T) {
        let s = &mut **self.scenes.last_mut().expect("updating empty scene stack");
        let t = s.update(ctx, state);
        self.switch(t);
    }

    pub fn draw(&mut self, ctx: &mut Context, state: &mut T) {
        SceneStack::draw_scenes(&mut self.scenes, ctx, state)
    }

    fn draw_scenes(scenes: &mut [Box<dyn Scene<T>>], ctx: &mut Context, state: &mut T) {
        if let Some((curr, rest)) = scenes.split_last_mut() {
            if curr.draw_previous() {
                SceneStack::draw_scenes(rest, ctx, state)
            }
            curr.draw(ctx, state);
        }
    }

    pub fn input(&mut self, _ctx: &mut Context, input: KeyInput, repeated: bool) {
        let s = &mut **self.scenes.last_mut().expect("updating empty scene stack");
        let t = s.key_down(input, repeated);
        self.switch(t);
    }
}

