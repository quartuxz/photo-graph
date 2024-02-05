use std::cmp;

use image::{GenericImageView, Pixel, Rgba, RgbaImage};

pub fn color_u8_to_f32(color: &Rgba<u8>)->Rgba<f32>{
    Rgba([color.0[0] as f32/255.0,color.0[1] as f32/255.0,color.0[2] as f32/255.0,color.0[3] as f32/255.0])
}

pub fn color_f32_to_u8(color: &Rgba<f32>)->Rgba<u8>{
    Rgba([(color.0[0]*255.0)as u8,(color.0[1]*255.0)as u8,(color.0[2]*255.0)as u8,(color.0[3]*255.0)as u8])
}

pub fn bilinear_interpolate(image:&RgbaImage,x:f64,y:f64)->Rgba<f32>{

    if x<0.0 || y<0.0||x>image.width()as f64 || y > image.height()as f64{
        return Rgba([0.0,0.0,0.0,0.0]);
    }

    let mut four_near : [(Rgba<f32>,(f64,f64));4] = [(Rgba([0.0,0.0,0.0,0.0]),(0.0,0.0));4];

    //locate each of the four nearest pixel centers and store their color and position.
    let mut i = 0;
    for yi in [-0.5,0.5]{
        for xi in [-0.5,0.5]{
            four_near[i].1.0 = x.round()+xi;
            four_near[i].1.1 = y.round()+yi;
            let xNear = four_near[i].1.0.floor() as u32;
            let yNear = four_near[i].1.1.floor() as u32;
            let pix = image.get_pixel_checked(xNear, yNear).unwrap_or(image.get_pixel(cmp::min(cmp::max(xNear,image.width()-1),0), cmp::min(cmp::max(yNear,image.height()-1),0)));
            four_near[i].0 = color_u8_to_f32(&pix);
            i+=1;
        }
    }



    //do the bilinear interpolation
    let near01To02 = saturating_add_rgba(&multiply_color(&four_near[0].0, ((four_near[1].1.0-x)/(four_near[1].1.0-four_near[0].1.0)) as f32),
                                                &multiply_color(&four_near[1].0, ((x-four_near[0].1.0)/(four_near[1].1.0-four_near[0].1.0)) as f32));
    let near03To04 = saturating_add_rgba(&multiply_color(&four_near[2].0, ((four_near[1].1.0-x)/(four_near[1].1.0-four_near[0].1.0)) as f32),
                                                &multiply_color(&four_near[3].0, ((x-four_near[0].1.0)/(four_near[1].1.0-four_near[0].1.0)) as f32));
    saturating_add_rgba(&multiply_color(&near01To02, ((four_near[3].1.1-y)/(four_near[3].1.1-four_near[0].1.1)) as f32),
                        &multiply_color(&near03To04, ((y-four_near[0].1.1)/(four_near[3].1.1-four_near[0].1.1)) as f32))

}

pub fn return_non_empty(image:&RgbaImage)->RgbaImage{
    if image.is_empty(){
        return RgbaImage::from_fn(500, 500, |x,y|{Rgba([100,0,50,255])});
    }
    image.clone()
}

pub fn inverse_of_add(foreground:&Rgba<f32>,background:&Rgba<f32>)->Rgba<f32>{
    let mut result = saturating_add_rgba(foreground, background);
    result.invert();
    result
}

pub fn screen_formula(foreground:&Rgba<f32>,background:&Rgba<f32>)->Rgba<f32>{
    let mut finverted = foreground.clone();
    finverted.invert();
    let mut binverted = background.clone();
    binverted.invert();
    let mut result = multiply_rgba_by_rgba(&finverted, &binverted);
    result.invert();
    result 
}

pub fn lighten_formula(foreground:&Rgba<f32>,background:&Rgba<f32>)->Rgba<f32>{
    let flightness = foreground.0[0] +foreground.0[1] +foreground.0[2];
    let blightness = background.0[0] +background.0[1] +background.0[2];
    if(flightness>blightness){
        foreground.clone()
    }else{
        background.clone()
    }
}

pub fn darken_formula(foreground:&Rgba<f32>,background:&Rgba<f32>)->Rgba<f32>{
    let flightness = foreground.0[0]+foreground.0[1] +foreground.0[2];
    let blightness = background.0[0] +background.0[1] +background.0[2] ;
    if(flightness>blightness){
        background.clone()

    }else{
        foreground.clone()
    }
}

pub fn color_dodge_formula(foreground:&Rgba<f32>,background:&Rgba<f32>)->Rgba<f32>{

    let mut finverted = foreground.clone();
    finverted.invert();
    divide_rgba_by_rgba(background, &finverted,true)
}

pub fn color_burn_formula(foreground:&Rgba<f32>,background:&Rgba<f32>)->Rgba<f32>{

    let mut binverted = background.clone();
    binverted.invert();
    let mut result = divide_rgba_by_rgba(&binverted, foreground,true);

    result.invert();
    result
}

pub fn divide_rgba_by_rgba(lhs:&Rgba<f32>,rhs:&Rgba<f32>, maxOnDivZero : bool)->Rgba<f32>{
    if maxOnDivZero{
        Rgba([(if(lhs.0[0]==0.0){0.0} else if rhs.0[0] != 0.0 {lhs.0[0]/rhs.0[0]}else{255.0}),(if(lhs.0[1]==0.0){0.0} else if rhs.0[1] != 0.0 {lhs.0[1]/rhs.0[1]}else{255.0}),(if(lhs.0[2]==0.0){0.0} else if rhs.0[2] != 0.0 {lhs.0[2]/rhs.0[2]}else{255.0}),(if(lhs.0[3]==0.0){0.0} else if rhs.0[3] != 0.0 {lhs.0[3]/rhs.0[3]}else{255.0}) ])
    }else{
        Rgba([(if(lhs.0[0]==0.0){0.0} else if rhs.0[0] != 0.0 {lhs.0[0]/rhs.0[0]}else{0.0}),(if(lhs.0[1]==0.0){0.0} else if rhs.0[1] != 0.0 {lhs.0[1]/rhs.0[1]}else{0.0}),(if(lhs.0[2]==0.0){0.0} else if rhs.0[2] != 0.0 {lhs.0[2]/rhs.0[2]}else{0.0}),(if(lhs.0[3]==0.0){0.0} else if rhs.0[3] != 0.0 {lhs.0[3]/rhs.0[3]}else{0.0}) ])
    }
    
}

pub fn blend<F:FnOnce(&Rgba<f32>,&Rgba<f32>)->Rgba<f32>>(foreground:&Rgba<u8>,background:&Rgba<u8>,blendMode:F)->Rgba<u8>{
    let balpha = normalized(background.0[3]);
    let f32Foreground = color_u8_to_f32(&foreground);
    let f32Background = color_u8_to_f32(&background);

    color_f32_to_u8(&saturating_add_rgba(&get_relative_color(&f32Foreground, 1.0-f32Background.0[3]),&get_relative_color(&blendMode(&f32Foreground,&f32Background), f32Background.0[3])))
}

pub fn saturating_add_rgba(lhs:&Rgba<f32>,rhs:&Rgba<f32>)->Rgba<f32>{
    Rgba([(lhs.0[0] + rhs.0[0]).min(1.0),(lhs.0[1] + rhs.0[1]).min(1.0),(lhs.0[2] + rhs.0[2]).min(1.0),(lhs.0[3] + rhs.0[3]).min(1.0)])
}



pub fn saturating_subtract_rgba(lhs:&Rgba<u8>,rhs:&Rgba<u8>)->Rgba<u8>{
    Rgba([lhs.0[0].saturating_sub(rhs.0[0]),lhs.0[1].saturating_sub(rhs.0[1]),lhs.0[2].saturating_sub(rhs.0[2]),lhs.0[3].saturating_sub(rhs.0[3])])
}



pub fn multiply_rgba_by_rgba(lhs:&Rgba<f32>,rhs:&Rgba<f32>)->Rgba<f32>{
    Rgba([lhs.0[0]*rhs.0[0],lhs.0[1]*rhs.0[1],lhs.0[2]*rhs.0[2],lhs.0[3]*rhs.0[3]])
}



pub fn get_relative_color(color:&Rgba<f32>,relative_to:f32)->Rgba<f32>{
    Rgba([color.0[0]*relative_to,color.0[1]*relative_to,color.0[2]*relative_to,color.0[3]])
}

pub fn multiply_color(color:&Rgba<f32>,value:f32)->Rgba<f32>{
    Rgba([color.0[0]*value,color.0[1]*value,color.0[2]*value,color.0[3]*value])
}

pub fn normalized(opacity:u8)->f32{
    f32::from(opacity)/255.0
}
