use image::{RgbaImage, Rgba, Pixel};

pub fn inverse_of_add(foreground:&Rgba<u8>,background:&Rgba<u8>)->Rgba<u8>{
    let mut result = saturating_add_rgba(foreground, background);
    result.invert();
    result
}

pub fn screen_formula(foreground:&Rgba<u8>,background:&Rgba<u8>)->Rgba<u8>{
    let mut finverted = foreground.clone();
    finverted.invert();
    let mut binverted = background.clone();
    binverted.invert();
    let mut result = multiply_rgba_by_rgba(&finverted, &binverted);
    result.invert();
    result 
}

pub fn lighten_formula(foreground:&Rgba<u8>,background:&Rgba<u8>)->Rgba<u8>{
    let flightness = foreground.0[0] as u32+foreground.0[1] as u32+foreground.0[2] as u32;
    let blightness = background.0[0] as u32+background.0[1] as u32+background.0[2] as u32;
    if(flightness>blightness){
        foreground.clone()
    }else{
        background.clone()
    }
}

pub fn darken_formula(foreground:&Rgba<u8>,background:&Rgba<u8>)->Rgba<u8>{
    let flightness = foreground.0[0] as u32+foreground.0[1] as u32+foreground.0[2] as u32;
    let blightness = background.0[0] as u32+background.0[1] as u32+background.0[2] as u32;
    if(flightness>blightness){
        background.clone()

    }else{
        foreground.clone()
    }
}

pub fn color_dodge_formula(foreground:&Rgba<u8>,background:&Rgba<u8>)->Rgba<u8>{

    let mut finverted = foreground.clone();
    finverted.invert();
    divide_rgba_by_rgba(background, &finverted,true)
}

pub fn color_burn_formula(foreground:&Rgba<u8>,background:&Rgba<u8>)->Rgba<u8>{

    let mut binverted = background.clone();
    binverted.invert();
    let mut result = divide_rgba_by_rgba(&binverted, foreground,true);

    result.invert();
    result
}

pub fn divide_rgba_by_rgba(lhs:&Rgba<u8>,rhs:&Rgba<u8>, maxOnDivZero : bool)->Rgba<u8>{
    if maxOnDivZero{
        Rgba([(if rhs.0[0] != 0 {f32::from(lhs.0[0])/(f32::from(rhs.0[0])/255.0)}else{255.0}).round() as u8,(if rhs.0[1] != 0 {f32::from(lhs.0[1])/(f32::from(rhs.0[1])/255.0)}else{255.0}).round() as u8,(if rhs.0[2] != 0 {f32::from(lhs.0[2])/(f32::from(rhs.0[2])/255.0)}else{255.0}).round() as u8,(if rhs.0[3] != 0 {f32::from(lhs.0[3])/(f32::from(rhs.0[3])/255.0)}else{255.0}).round() as u8 ])
    }else{
        Rgba([(if rhs.0[0] != 0 {f32::from(lhs.0[0])/(f32::from(rhs.0[0])/255.0)}else{0.0}).round() as u8,(if rhs.0[1] != 0 {f32::from(lhs.0[1])/(f32::from(rhs.0[1])/255.0)}else{0.0}).round() as u8,(if rhs.0[2] != 0 {f32::from(lhs.0[2])/(f32::from(rhs.0[2])/255.0)}else{0.0}).round() as u8,(if rhs.0[3] != 0 {f32::from(lhs.0[3])/(f32::from(rhs.0[3])/255.0)}else{0.0}).round() as u8 ])
    }
    
}

pub fn blend<F:FnOnce(&Rgba<u8>,&Rgba<u8>)->Rgba<u8>>(foreground:&Rgba<u8>,background:&Rgba<u8>,blendMode:F)->Rgba<u8>{
    let balpha = normalized(background.0[3]);
    saturating_add_rgba(&get_relative_color(foreground, 1.0-balpha),&get_relative_color(&blendMode(foreground,background), balpha))
}

pub fn saturating_add_rgba(lhs:&Rgba<u8>,rhs:&Rgba<u8>)->Rgba<u8>{
    Rgba([lhs.0[0].saturating_add(rhs.0[0]),lhs.0[1].saturating_add(rhs.0[1]),lhs.0[2].saturating_add(rhs.0[2]),lhs.0[3].saturating_add(rhs.0[3])])
}



pub fn saturating_subtract_rgba(lhs:&Rgba<u8>,rhs:&Rgba<u8>)->Rgba<u8>{
    Rgba([lhs.0[0].saturating_sub(rhs.0[0]),lhs.0[1].saturating_sub(rhs.0[1]),lhs.0[2].saturating_sub(rhs.0[2]),lhs.0[3].saturating_sub(rhs.0[3])])
}



pub fn multiply_rgba_by_rgba(lhs:&Rgba<u8>,rhs:&Rgba<u8>)->Rgba<u8>{
    Rgba([((f32::from(lhs.0[0])/255.0)*f32::from(rhs.0[0])).round() as u8,((f32::from(lhs.0[1])/255.0)*f32::from(rhs.0[1])).round() as u8,((f32::from(lhs.0[2])/255.0)*f32::from(rhs.0[2])).round() as u8,((f32::from(lhs.0[3])/255.0)*f32::from(rhs.0[3])).round() as u8])
}



pub fn get_relative_color(color:&Rgba<u8>,relative_to:f32)->Rgba<u8>{
    Rgba([(f32::from(color.0[0])*relative_to).round() as u8,(f32::from(color.0[1])*relative_to).round() as u8,(f32::from(color.0[2])*relative_to).round() as u8,color.0[3]])
}

pub fn multiply_color(color:&Rgba<u8>,value:f32)->Rgba<u8>{
    Rgba([(f32::from(color.0[0])*value).round() as u8,(f32::from(color.0[1])*value).round() as u8,(f32::from(color.0[2])*value).round() as u8,(f32::from(color.0[3])*value).round() as u8])
}

pub fn normalized(opacity:u8)->f32{
    f32::from(opacity)/255.0
}
