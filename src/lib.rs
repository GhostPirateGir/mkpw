extern crate pancurses;
extern crate md_5;
extern crate base64;
extern crate rand;

use rand::Rng;
use md_5::{Md5, Digest};

pub struct Mkpw {
    window: pancurses::Window,
    domain: Option<String>,
    confirmation: bool,
    urlsafe: bool,
    pwlen: usize,
}

static MASKLEN: i32 = 11;

impl Mkpw {
    fn print_center(&self, outstr: &String, prepare_prompt: bool) {
        let (y, x) = self.window.get_max_yx();
    	self.window.clear();
        self.window.mv(y/2 - 1, x/2 - (outstr.len() / 2 - 1) as i32);
        self.window.printw(outstr);
        if prepare_prompt {
            self.window.mv(y/2, x/2);
        }
        self.window.refresh();
    }

    fn gen_pwmask() -> String {
        let mut mask = String::new();
        let mut rng = rand::thread_rng();

        for _ in 0..MASKLEN {
                if rng.gen() {
                mask.push('O');
            } else {
                mask.push('o');
            }
        }
        mask
    }

    fn read_passphrase_prompt(&self, prompt: &String) -> String {
        let mut passphrase = String::new();
    
        self.print_center(prompt, true);
        self.window.mv(self.window.get_cur_y(), self.window.get_cur_x() - (MASKLEN / 2 - 1));
        pancurses::cbreak();
        'read: loop {
            if let Some(c) = self.window.getch() {
                match c {
                    pancurses::Input::Character('\n') => break 'read,
                    //pancurses::Input::Character('\x08') => { passphrase.pop(); },
                    pancurses::Input::Character(c) => { 
                        passphrase.push(c);
                        self.window.printw(&Mkpw::gen_pwmask());
                        self.window.refresh();
                        self.window.mv(self.window.get_cur_y(), self.window.get_cur_x() - MASKLEN);
                    },
                    _ => continue,
                }
            } else {
                panic!("Error reading from terminal!");
            }
        }
    
        self.window.clear();
        self.window.refresh();
        String::from(passphrase.trim())
    }

    fn write(&self, outstr: &String) {
        self.print_center(outstr, false);
	    self.window.getch();
    }
    
    fn read_passphrase(&self) -> String {
        let prompt = String::from("Enter passphrase: ");
        self.read_passphrase_prompt(&prompt)
    }
    
    fn read_passphrase_confirm(&self) -> String {
        let prompt = String::from("Confirm passphrase: ");
        self.read_passphrase_prompt(&prompt)
    }

    fn generate_pw(&self, passphrase: &String) -> String {
        //let dgst = base64::encode(&Md5::digest(&format!("{}:{}", self.domain.as_ref().unwrap(), &passphrase).as_bytes()));
        let dgst = Md5::digest(&format!("{}:{}", self.domain.as_ref().unwrap(), &passphrase).as_bytes());
        let mut hash: String;
        if self.urlsafe {
            hash = base64::encode_config(&dgst, base64::URL_SAFE);
        } else {
            hash = base64::encode(&dgst);
        }
        hash.split_off(self.pwlen);
        hash
    }

    pub fn new(mut args: std::env::Args) -> Result<Mkpw, String> {
        let window = pancurses::initscr();
        pancurses::curs_set(0);
        pancurses::noecho();

        // Default values
        let mut domain: Option<String> = None;
        let mut confirmation = true;
        let mut urlsafe = false;
        let mut lenres: Result<usize, String> = Ok(10);

        // skip program name
        args.next();
        let mut arg = args.next();
        while arg != None {
            match arg.unwrap().as_ref() {
                "-n" => confirmation = false,
                "-a" => urlsafe = true,
                "-l" => {
                    let len: Option<String> = args.next();
                    if len == None {
                        lenres = Err(String::from("Option -l requires a length parameter!"))
                    } else {
                        let lenunwrap = len.unwrap();
                        let parselen = lenunwrap.parse::<usize>();
                        if parselen.is_ok() {
                            lenres = Ok(parselen.ok().unwrap());
                        } else {
                            lenres = Err(format!("Failed to parse parameter '{}' to option -l as integer value: {}", lenunwrap, parselen.err().unwrap()))
                        }
                    }
                }
                //_ => domain = Some(String::from(arg.unwrap().trim())),
                x => domain = Some(String::from(x)),
            }
            arg = args.next();
        }

        if domain == None {
            Err(String::from("Domain missing!"))
        } else if lenres.is_err() {
            Err(lenres.err().unwrap())
        } else {
            Ok(Mkpw {
                window: window,
                domain: domain,
                confirmation: confirmation,
                urlsafe: urlsafe,
                pwlen: lenres.unwrap(),
            })
        }
    }

    pub fn run(&self) {
        let passphrase = self.read_passphrase();
        if self.confirmation {
            let p_confirm = self.read_passphrase_confirm();

            if passphrase != p_confirm {
                self.write(&String::from("Passphrase and confirmation do not match!"));
                Mkpw::exit();
            }
        }
        self.write(&format!("Password for domain '{}': {}", self.domain.as_ref().unwrap(), &self.generate_pw(&passphrase)));

        Mkpw::exit();
    }

    pub fn exit() {
        pancurses::endwin();
    }
}
