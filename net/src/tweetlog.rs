
use std::{
    string::*,
    sync::{mpsc, Mutex, RwLock},
    collections::{HashMap, HashSet},
};

/// Type for holding a map of 'logger_name -> Tweetlog'
type TweetLoggerMap = HashMap<String, Mutex<Tweetlog>>;

#[warn(non_camel_case_types)]
pub type listenerCallback = fn(LogLevel, Option<&str>, &str);



/// this is the actual memory space for our loggers
#[warn(non_upper_case_globals)]
lazy_static! {
//    pub(crate) static ref TWEET_LOGGER_MAP: RwLock<Tweetlog> =
//        RwLock::new(HashMap::new());

    pub static ref g_tweetlog: RwLock<Tweetlog> = RwLock::new(Tweetlog::new());
    //pub static ref g_tweetlog: Tweetlog = Tweetlog::new();
}


#[derive(Debug, Clone)]
pub enum LogLevel {
    Trace = 1,
    Debug,
    Info,
    Warning,
    Error,
}


impl From<char> for LogLevel {
    fn from(letter: char) -> Self {
        //assert!(letter.len() == 1);
        match letter {
            't' => LogLevel::Trace,
            'd' => LogLevel::Debug,
            'i' => LogLevel::Info,
            'w' => LogLevel::Warning,
            'e' => LogLevel::Error,
            _ => unreachable!(),
        }
    }
}
impl LogLevel {
    pub fn to_char(level: &LogLevel) -> char {
        match level {
            LogLevel::Trace => 't',
            LogLevel::Debug => 'd',
            LogLevel::Info => 'i',
            LogLevel::Warning => 'w',
            LogLevel::Error => 'e',
        }
    }

    pub fn as_char(&self) -> char {
        LogLevel::to_char(self)
    }
}


#[derive(Debug)]
struct Logger {
    pub level: LogLevel,
    pub callbacks: HashSet<listenerCallback>,
}

impl Logger {
    pub fn new() -> Self {
        Logger::with_level(LogLevel::Info)
    }

    pub fn with_level(level: LogLevel) -> Self {
        Logger {
            level,
            callbacks: HashSet::new(),
        }
    }
}

pub struct Tweetlog {
    log_by_tag: HashMap<String, Logger>,
}

impl Tweetlog {
    pub fn new() -> Self {
        let mut tlog = Tweetlog {
            log_by_tag: HashMap::new(),
        };
        tlog.log_by_tag.insert("_".to_string(), Logger::new());
        tlog
    }
}


impl Tweetlog {
    pub fn add(&mut self, tag: &str) /* -> Self */ {
        self.log_by_tag.insert(tag.to_string(), Logger::new());
    }

    // Setting the logging level, either globally, or per-tag
    pub fn set(&mut self, level: LogLevel, maybe_tag: Option<String>) {
        let tag = match maybe_tag {
            None => "_".to_string(),
            Some(tag) => tag,
        };
        // update existing logger
        {
            let maybe_logger = self.log_by_tag.get_mut(&tag);
            if let Some(logger) = maybe_logger {
                logger.level = level;
                return;
            };
        }
        // otherwise create new one
        self.log_by_tag.insert(tag, Logger::with_level(level));
    }

//    pub fn resetLevels(&mut self) {
//        for (_, mut logger) in self.log_by_tag {
//            logger.level = LogLevel::Info;
//        }
//    }

    pub fn listen(&mut self, cb: listenerCallback) {
        self.listen_to_tag("_", cb);
    }

    pub fn listen_to_tag(&mut self, tag: &str, cb: listenerCallback) {
        let logger = self.log_by_tag.get_mut(tag).unwrap();
        logger.callbacks.insert(cb);
    }

//    // Clear any registered log listeners or levels
//    pub fn unlistenAll(&mut self) {
//        for (_, mut logger) in self.log_by_tag {
//            logger.callbacks.clear();
//        }
//    }

    // Clear any registered log listeners or levels
    pub fn unlisten(&mut self, tag: &str) {
        let maybe_logger = self.log_by_tag.get_mut(tag);
        if let Some(logger) = maybe_logger {
            logger.callbacks.clear();
        }
    }

    // Check if a given level and tag would be logged
    pub fn should(&self, level: LogLevel, maybe_tag: Option<String>) -> bool {
        let tag = match maybe_tag {
            None => "_".to_string(),
            Some(tag) => tag,
        };
        let maybe_logger = self.log_by_tag.get(&tag);
        match maybe_logger {
            None => false,
            Some(logger) => (logger.level.clone() as usize) <= (level as usize),
        }
    }

    // callback according to level and tag
    fn tweet(&self, level: LogLevel, maybe_tag: Option<&str>, msg: &str) {
        // replace None to "_"
        let tag = match maybe_tag {
            None => "_",
            Some(tag) => tag,
        };
        // Find logger, if unknown tag use general
        let maybe_logger = self.log_by_tag.get(tag);
        // println!("maybe_logger({}) = {:?}", tag, maybe_logger);
        let logger = match maybe_logger {
            None => self.log_by_tag.get("_").unwrap(),
            Some(logger) => logger,
        };
        // print if logger can
        if (logger.level.clone() as usize) <= (level.clone() as usize) {
            for cb in logger.callbacks.clone() {
                cb(level.clone(), Some(tag), msg);
            }

        }

    }

    // -- sugar -- //

    pub fn t(&self, msg: &str) {
        self.tweet(LogLevel::Trace, None, msg);
    }
    pub fn tt(&self, tag: &str, msg: &str) {
        self.tweet(LogLevel::Trace, Some(tag), msg);
    }

    pub fn d(&self, msg: &str){
        self.tweet(LogLevel::Debug, None, msg);
    }
    pub fn dd(&self, tag: &str, msg: &str) {
        self.tweet(LogLevel::Debug, Some(tag), msg);
    }

    pub fn i(&self, msg: &str){
        self.tweet(LogLevel::Info, None, msg);
    }
    pub fn ii(&self, tag: &str, msg: &str) {
        self.tweet(LogLevel::Info, Some(tag), msg);
    }

    pub fn w(&self, msg: &str){
        self.tweet(LogLevel::Warning, None, msg);
    }
    pub fn ww(&self, tag: &str, msg: &str) {
        self.tweet(LogLevel::Warning, Some(tag), msg);
    }

    pub fn e(&self, msg: &str){
        self.tweet(LogLevel::Error, None, msg);
    }
    pub fn ee(&self, tag: &str, msg: &str) {
        self.tweet(LogLevel::Error, Some(tag), msg);
    }

    // -- provided listeners -- //

    pub fn console(level: LogLevel, maybe_tag: Option<&str>, msg: &str) {
        match maybe_tag {
            None      => println!("(global)[{}]  {}", level.as_char(), msg),
            Some(tag) => println!("({:?})[{}]  {}", tag, level.as_char(), msg),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_default_should() {
        let tweetlog = Tweetlog::new();

        assert!(!tweetlog.should(LogLevel::Trace, None));
        assert!(!tweetlog.should(LogLevel::Debug, None));
        assert!(tweetlog.should(LogLevel::Info, None));
        assert!(tweetlog.should(LogLevel::Warning, None));
        assert!(tweetlog.should(LogLevel::Error, None));
    }

    #[test]
    fn log_should() {
        let mut tweetlog = Tweetlog::new();
        tweetlog.set(LogLevel::Error, None);

        assert!(!tweetlog.should(LogLevel::Trace, None));
        assert!(!tweetlog.should(LogLevel::Debug, None));
        assert!(!tweetlog.should(LogLevel::Info, None));
        assert!(!tweetlog.should(LogLevel::Warning, None));
        assert!(tweetlog.should(LogLevel::Error, None));

        tweetlog.set(LogLevel::Trace, None);

        assert!(tweetlog.should(LogLevel::Trace, None));
        assert!(tweetlog.should(LogLevel::Debug, None));
        assert!(tweetlog.should(LogLevel::Info, None));
        assert!(tweetlog.should(LogLevel::Warning, None));
        assert!(tweetlog.should(LogLevel::Error, None));

    }

    #[test]
    fn log_should_tag() {
        let mut tweetlog = Tweetlog::new();
        tweetlog.set(LogLevel::Error, Some("toto".to_string()));

        assert!(!tweetlog.should(LogLevel::Trace, None));
        assert!(!tweetlog.should(LogLevel::Debug, None));
        assert!(tweetlog.should(LogLevel::Info, None));
        assert!(tweetlog.should(LogLevel::Warning, None));
        assert!(tweetlog.should(LogLevel::Error, None));
    }

    #[test]
    fn log_println_hello() {
        let mut tweetlog = Tweetlog::new();
        tweetlog.add("errorlog");

        // set general logging to error only
        tweetlog.set(LogLevel::Warning, None);
        tweetlog.listen(Tweetlog::console);

        // set testlogger output to trace level
        tweetlog.add("tracelog");
        tweetlog.set(LogLevel::Trace, Some("tracelog".to_string()));
        tweetlog.listen_to_tag("tracelog", Tweetlog::console);

        tweetlog.t("hello trace");
        tweetlog.d("hello debug");
        tweetlog.i("hello info");
        tweetlog.w("hello warning");
        tweetlog.e("hello error");

    }
}