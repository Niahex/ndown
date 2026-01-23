use makepad_widgets::*;

pub struct EventManager {
    pub repeat_timer: Timer,
    pub repeat_key: Option<KeyCode>,
    pub last_click_time: f64,
    pub last_click_pos: Option<DVec2>,
}

impl Default for EventManager {
    fn default() -> Self {
        Self {
            repeat_timer: Timer::default(),
            repeat_key: None,
            last_click_time: 0.0,
            last_click_pos: None,
        }
    }
}

impl EventManager {
    pub fn start_key_repeat(&mut self, cx: &mut Cx, key: KeyCode) {
        self.repeat_key = Some(key);
        self.repeat_timer = cx.start_timeout(0.5); // 500ms initial delay
    }
    
    pub fn stop_key_repeat(&mut self, cx: &mut Cx) {
        if self.repeat_key.is_some() {
            cx.stop_timer(self.repeat_timer);
            self.repeat_key = None;
        }
    }
    
    pub fn is_double_click(&mut self, pos: DVec2, time: f64) -> bool {
        let is_double = if let Some(last_pos) = self.last_click_pos {
            time - self.last_click_time < 0.5 && 
            (pos - last_pos).length() < 10.0
        } else {
            false
        };
        
        self.last_click_time = time;
        self.last_click_pos = Some(pos);
        
        is_double
    }
}
