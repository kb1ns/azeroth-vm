
pub struct ClassArena {
    pub cp: super::Classpath,

}

pub struct Klass {

}

enum Classloader {
    ROOT,
    EXT,
    // TODO classloader instance
    APP,
}

impl ClassArena {

    pub fn define_class(&self, class_name: &str) {
    }
}
