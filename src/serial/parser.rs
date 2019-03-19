/// Parse received data from power controller board.
/// Data format using NMEA 0183 data specification.
///
/// 数据帧以'$'为起始字符，以<LF>(即'\n')为结束字符，','为字段分割符，*字符后接校验位。
/// 1.开关量'x':以'1'表示开关闭合，'0'表示开关断开。
///   数字量'x':表示'0'-'9'的数字。
/// 2.控制板循环发送以下状态：
/// (1)输入开关量状态：
///   "IN1:x1;IN2:x2;IN3:x3;ACI:x4;"
/// 说明：分别对应三个输入和市电有无
/// 消息格式："$IN,x1,x2,x3,x4*c\n"，c为校验位，由两个十六进制数字字符构成。(校验位可选?)
/// 例：如发送数据"IN1:1;IN2:0;IN3:1;AC:1;"，则对应数据帧为"$IN,1,0,1,1\n"
/// (2)输出开关量状态：
///   "NO1:x1;NO2:x2;NO3:x3;NO4:x4;NC5:x5;NC6:x6;\n"
/// 说明：前6项分别对应6个输出。
/// 对应消息格式："$NO,x1,x2,x3,x4,x5,x6*c\n"
/// (3)UPS电源输入状态：
///   "UPS V:xxxV;UPS I:xx.xA;UPS P:xxxW;UPS OV:x;\n"
/// 说明：分别对应UPS的电压，电流，功率，超压否。
/// 对应消息格式："$UPS,xxx,xx.x,xxx,x*c\n"
/// (4)电池状态：
///   "BT V:xx.xV;BT I:+xx.xA;BT C:xx%;\n"
/// 说明：分别对应电池电压，电流，容量百分比。
/// 对应消息格式："$BT,xx.x,xx.x,xx*c\n"
/// (5)输出直流状态：
///   "DC V:xx.xV;DC1 I:xx.xA;DC2 I:xx.xA;\n"
/// 说明：分别对应输出电压，输出1电流，输出2电流
/// 对应消息格式："$DC,xx.x,xx.x,xx.x*c\n"

// Assembly message in message cache with serial port buffer
pub fn get_msg<'a, 'b>(buffer: &'a [u8], msg_cache: &'b mut Vec<u8>) -> Option<&'b [u8]> {
    if msg_cache.len() > 0 && msg_cache[msg_cache.len() - 1] == b'\n' {
        msg_cache.truncate(0);
    }
    if buffer.len() > 0 {
        for x in buffer.iter() {
            if *x == b'$' {
                msg_cache.truncate(0);
                msg_cache.push(*x);
            } else {
                if msg_cache.len() == 0 {
                    break;
                }
                if *x == b'\n' {
                    msg_cache.push(*x);
                    break;
                }
                msg_cache.push(*x)
            }
        }
    }
    if msg_cache.len() > 2 && msg_cache[0] == b'$' && msg_cache[msg_cache.len() - 1] == b'\n' {
        Some(&msg_cache[..])
    } else {
        None
    }
}
