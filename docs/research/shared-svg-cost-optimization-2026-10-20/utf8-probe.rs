use std::{env,hint::black_box,time::{Instant,Duration}};
fn round(x:f64)->f64{let f=x.floor();(if x-f>=0.5{f+1.0}else{f}).copysign(x)}
#[inline]
fn num_digits(value: f64, buf: &mut [u8; 32]) -> Option<usize> {
    let n = round(value * 100.0);
    if n == 0.0 {
        buf[31] = b'0';
        return Some(31);
    }
    let (neg, m) = if n > 0.0 {
        if n < 1e11 {
            (false, n)
        } else {
            return None;
        }
    } else if n > -1e11 {
        (true, -n)
    } else {
        return None;
    };
    // The rounded magnitude is an exact integer below 1e11. Write it once from a stack
    // buffer, avoiding nested Formatter calls and tiny String growth steps.
    let cents = m as u64;
    let mut integer = cents / 100;
    let fraction = (cents % 100) as u8;
    let mut start = buf.len();
    if fraction != 0 {
        if fraction % 10 != 0 {
            start -= 1;
            buf[start] = b'0' + fraction % 10;
        }
        start -= 1;
        buf[start] = b'0' + fraction / 10;
        start -= 1;
        buf[start] = b'.';
    }
    loop {
        start -= 1;
        buf[start] = b'0' + (integer % 10) as u8;
        integer /= 10;
        if integer == 0 {
            break;
        }
    }
    if neg {
        start -= 1;
        buf[start] = b'-';
    }
    Some(start)
}

fn batch(mode:&str,values:&[f64])->usize{
 let mut out=String::with_capacity(64*values.len());
 for &value in values{let mut buf=[0u8;32];let start=num_digits(value,&mut buf).unwrap();
 if mode=="tail"{out.push_str(std::str::from_utf8(&buf[start..]).unwrap());}
 else{out.push_str(&std::str::from_utf8(&buf).unwrap()[start..]);}}
 black_box(out).len()
}
fn main(){let mode=env::args().nth(1).unwrap();let mut seed=0x16a57u64;
 let values:Vec<_>=(0..4096).map(|i|{seed=seed.wrapping_mul(6364136223846793005).wrapping_add(1);let n=(seed>>32)%10000000;let n=n as f64/100.;if i%2==0{n}else{-n}}).collect();
 for _ in 0..5{batch(&mode,&values);}
 let mut reps=1;loop{let at=Instant::now();for _ in 0..reps{black_box(batch(&mode,&values));}if at.elapsed()>=Duration::from_millis(10){break;}reps*=2;}
 let mut times=Vec::new();let mut total=Duration::ZERO;while times.len()<9 || total<Duration::from_millis(300){let at=Instant::now();for _ in 0..reps{black_box(batch(&mode,&values));}let duration=at.elapsed();total+=duration;times.push(duration.as_secs_f64()*1e9/(reps as f64*values.len() as f64));}
 times.sort_by(f64::total_cmp);println!("{{\"mode\":\"{}\",\"samples\":{},\"nsPerNumber\":{}}}",mode,times.len(),times[(times.len()-1)/2]);}
