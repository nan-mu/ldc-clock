use core::fmt::Display;

use alloc::vec::Vec;

pub static mut NOW: DateTime = DateTime {
    hour: (0, 0),
    min: (0, 0),
    sec: (0, 0),
};

pub struct DateTime {
    pub hour: (i8, i8),
    pub min: (i8, i8),
    pub sec: (i8, i8),
}

pub enum UpdateIndex {
    Sec1,
    Sec10,
    Min1,
    Min10,
    Hour1,
    Hour10,
}

impl DateTime {
    pub fn sub_sec(&mut self) -> Vec<UpdateIndex> {
        let mut ans = Vec::new();
        ans.push(UpdateIndex::Sec1);
        let sec = self.sec.0 * 10 + self.sec.1 - 1;
        if sec < 0 {
            self.sec = (5, 9);
            ans.push(UpdateIndex::Sec10);
            ans.push(UpdateIndex::Min1);
            let min = self.min.0 * 10 + self.min.1 - 1;
            if min < 0 {
                self.min = (5, 9);
                ans.push(UpdateIndex::Min10);
                ans.push(UpdateIndex::Hour1);
                let hour = self.hour.0 * 10 + self.hour.1 - 1;
                if hour < 0 {
                    self.hour = (2, 3);
                    ans.push(UpdateIndex::Hour10);
                } else {
                    if hour / 10 != self.hour.0 {
                        self.hour.0 -= 1;
                        self.hour.1 = 9;
                    } else {
                        self.hour.1 -= 1;
                    }
                }
            } else {
                if min / 10 != self.min.0 {
                    ans.push(UpdateIndex::Min10);
                    self.min.0 -= 1;
                    self.min.1 = 9;
                } else {
                    self.min.1 -= 1;
                }
            }
        } else {
            if sec / 10 != self.sec.0 {
                ans.push(UpdateIndex::Sec10);
                self.sec.0 -= 1;
                self.sec.1 = 9;
            } else {
                self.sec.1 -= 1;
            }
        }
        ans
    }

    pub fn add_sec(&mut self) -> Vec<UpdateIndex> {
        //8位，使用6位，分别表示数字是否发生变化
        let mut ans = Vec::new(); //秒的个位需要变化
        ans.push(UpdateIndex::Sec1);
        let sec = self.sec.0 * 10 + self.sec.1 + 1;
        if sec >= 60 {
            //更新分
            self.sec = (0, 0);
            //分的个位，秒的10位发生变化
            ans.push(UpdateIndex::Sec10);
            ans.push(UpdateIndex::Min1);
            let min = self.min.0 * 10 + self.min.1 + 1;
            if min >= 60 {
                //更新时
                self.min = (0, 0);
                //分的10位，时的个位发生变化
                ans.push(UpdateIndex::Min10);
                ans.push(UpdateIndex::Hour1);
                let hour = self.hour.0 * 10 + self.hour.1 + 1;
                if hour >= 24 {
                    self.hour = (0, 0);
                    ans.push(UpdateIndex::Hour10);
                } else {
                    //不更新时
                    if hour / 10 != self.hour.0 {
                        // 10位发生变化
                        self.hour.0 += 1;
                        self.hour.1 = 0;
                    } else {
                        //10位不发生变化
                        self.hour.1 += 1;
                    }
                }
            } else {
                //不更新时
                if min / 10 != self.min.0 {
                    // 10位发生变化
                    ans.push(UpdateIndex::Min10);
                    self.min.0 += 1;
                    self.min.1 = 0;
                } else {
                    //10位不发生变化
                    self.min.1 += 1;
                }
            }
        } else {
            //不更新分
            if sec / 10 != self.sec.0 {
                // 10位发生变化
                ans.push(UpdateIndex::Sec10);
                self.sec.0 += 1;
                self.sec.1 = 0;
            } else {
                //10位不发生变化
                self.sec.1 += 1;
            }
        }
        ans
    }
}

impl DateTime {
    pub fn build(&mut self, value: &[u8]) {
        let hour = value[0] + ((value[1] + (value[2] + 13 >= 60) as u8) >= 60) as u8 % 24;
        let hour = hour as i8;
        self.hour = (hour / 10, hour % 10);
        // 这里加13秒是为了中和编译烧录时间
        let min = (value[1] + (value[2] + 13 >= 60) as u8) % 60;
        let min = min as i8;
        self.min = (min / 10, min % 10);
        let sec = (value[2] + 13) % 60;
        let sec = sec as i8;
        self.sec = (sec / 10, sec % 10);
    }
}

impl Display for DateTime {
    /// 该函数不会主动更新时间，不应该在显示时间时使用，这里仅作为调试输出到控制台
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}",
            self.hour.0 * 10 + self.hour.1,
            self.min.0 * 10 + self.min.1,
            self.sec.0 * 10 + self.sec.1
        )
    }
}
