use std::sync::{Arc, Mutex};

use diesel::{Connection, RunQueryDsl, SqliteConnection};
use rppal::gpio::*;

use crate::config::DATABASE_URL;
use crate::models::Output;
use std::thread::sleep;
use std::time::Duration;

const DS: u8 = 2;
//to pin14
const SHCP: u8 = 3;
// to pin11
const STCP: u8 = 4; //to pin12

lazy_static! {
 pub static ref GPIO : Arc<Mutex<Io>> = {
        let conn = SqliteConnection::establish(DATABASE_URL).unwrap();
        let mut io = Io {
         output: 0,
         input: 0,
         conn,
        };
        io.init();
        Arc::new(Mutex::new(io))
    };
}

pub struct Io {
    pub output: u128,
    pub input: u128,
    conn: SqliteConnection,
}

impl Io {
    pub fn init(&mut self) {
        use crate::schema::output::dsl::*;
        let outputs: Vec<Output> = output.load::<Output>(&self.conn).expect("error");
        let mut op = 0u128;
        for x in outputs {
            op += x.state as u128 * (1u128 << x.address as u128)
        }
        self.output = op
    }


    pub fn flash() -> Result<()> {
        Ok(())
    }

    pub fn flash_out(self) -> Result<()> {
        let gpio = Gpio::new()?;
        let mut ds_pin = gpio.get(DS)?.into_output();
        let mut st_pin = gpio.get(STCP)?.into_output();
        let mut sh_pin = gpio.get(SHCP)?.into_output();
        let mut output = self.output.clone();
        let array = &mut [0u8; 128];
        for x in 0usize..128 {
            let mut i = 1u128;
            i &= output;
            output >>= 1;
            array[x] = i as u8;
        }
        array.reverse();
        let mut address = 0u8;
        for x in array.iter() {
            if *x == 0 {
                ds_pin.set_low()
            } else {
                ds_pin.set_high()
            }
            sleep(Duration::from_micros(100));
            sh_pin.set_high();
            sleep(Duration::from_micros(100));
            sh_pin.set_low();
            sleep(Duration::from_micros(100));
            address += 1;
        }
        sleep(Duration::from_micros(100));
        st_pin.set_high();
        sleep(Duration::from_micros(100));
        st_pin.set_low();
        Ok(())
    }

    pub fn output_and_flash(&mut self, address: u8, state: u8) {
        let mut op = self.output;
        op = op.rotate_right(address as u32);
        op = (op >> 1 << 1) + state as u128;
        op = op.rotate_left(address as u32);
        self.output = op;
        Io.flash_out();
        ()
    }
}


