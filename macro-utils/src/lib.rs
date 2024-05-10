use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

//must provide a conversion_type attribute 
#[proc_macro_derive(TryFrom, attributes(conversion_type))]
pub fn derive_TryFrom(input: TokenStream) -> TokenStream {
    let parsed: DeriveInput = parse_macro_input!(input);
    let DeriveInput{ident,..} = parsed;
    let mut output = quote!();
    if(parsed.attrs.is_empty()){
        panic!("did not provide conversion_type argument!");
    }
    if let syn::Meta::List(arguments) = &parsed.attrs[0].meta{
        let conversion_type = arguments.tokens.clone();
        let mut conversionArms = quote!();
        if let syn::Data::Enum(parsedEnum) = parsed.data{
            for variant in parsedEnum.variants{
                
                let variant_ident = variant.ident;
                conversionArms = quote!(#conversionArms x if x == #ident::#variant_ident as #conversion_type => Ok(#ident::#variant_ident),);
            }
        }else{
            panic!("not an enum!")
        }
        output = quote! {
            impl TryFrom<#conversion_type> for #ident {
                type Error = ();
    
            
                fn try_from(v: #conversion_type) -> Result<Self, Self::Error> {
                    match v {
                        #conversionArms
                        _ => Err(()),
                    }
                }
            }
        };
    }else{
        panic!("no appropriate conversion_type argument(try conversion_type(type)) !");
    }

    output.into()
}

//highly custom macro
#[proc_macro]
pub fn make_blend(input: TokenStream) -> TokenStream{
    let func:proc_macro2::TokenStream = input.into();
    let output = quote!(
        Arc::get_mut(&mut self.buffer).unwrap().as_mut_rgba8().unwrap().enumerate_pixels_mut().for_each(|(x,y,pixel)|{
            let mut fpix = match foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
            let mut bpix = match background.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};

            
            *pixel = blend(&fpix, &bpix, #func);
            pixel.0[3] = fpix.0[3];
    }););
    output.into()
}


