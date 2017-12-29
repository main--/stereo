#[macro_use] extern crate cstr_macro;
extern crate widestring;

mod native {
    #![allow(dead_code)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

macro_rules! wrap_ptr {
    ($eptr:expr, $wrap:ident) => (
        {
            let ptr = $eptr ;
            if ptr.is_null() {
                None
            } else {
                Some ( $wrap ::from_raw ( ptr ) )
            }
        }
    )
}


macro_rules! gc_ret {
    ( $( pub fn $fname:ident ( $( $pname:ident : $ptype:ty ),* ) -> $ret:ty $body:block )+ ) => {
        $(
            pub fn $fname <S: ::safety::GcPtrStrategy < $ret >> (
                $( $pname : $ptype ),* , strat: &S
                    ) -> S::Target {
                strat.wrap( $body )
            }
        )+
    }
}


pub mod managed;
pub mod metadata;
pub mod safety;
pub mod runtime;


use managed::array::*;
use managed::object::*;
use managed::*;
use metadata::*;
use safety::*;
use runtime::*;

fn main() {
    println!("lets test ergonomics");

    let strat = unsafe { StackRefs::i_promise_to_never_store_references_anywhere_other_than_the_stack() };
    //let strat = GcHandles;

    let mono = Mono::init().unwrap();
    {
        let image = mono.open_image("/tmp/file.exe").unwrap();
        mono.root_domain().load_assembly(&image).unwrap();

        let class = image.class_from_name(Some("MyNS"), "Test").unwrap();
        let main = class.methods().find(|x| x.name() == "Main").unwrap();
        println!("main: {:?}", main);
        let args = ObjectArray::from_iter::<_, MonoString, _>(mono.root_domain(),
                                          &mono.class_string(),
                                          &[
                                              /*
                                              Some(MonoString::empty(mono.root_domain(), &strat).downcast()),
                                              Some(MonoString::new("yay", mono.root_domain(), &strat).downcast()),
                                               */
                                              MonoString::empty(mono.root_domain(), &strat),
                                              MonoString::new("yay", mono.root_domain(), &strat),
                                              ],
                                          &strat);
        /*
        let mainargs = ObjectArray::from_iter(mono.root_domain(),
                                              &mono.class_object(),
                                              &[args]);
        let result = main.invoke_array(Null, &mainargs);
         */

        println!("{:?}", &*args);


        let result = main.invoke(None, &[MonoValue::ObjectRef(Some(args.downcast())/*.into()*/)], &strat);
        let result = result.unwrap().unwrap();
        let result: i32 = *Boxed::cast(&result);
        println!("{:?}", result);
    }

    // ??????
    std::mem::forget(mono);
}

/*
fn main() {
    let mono = Mono::init().unwrap();
    doit(&mono);
    //let mono = Mono::init().unwrap();
    doit(&mono);
    doit(&mono);
}

fn doit(mono: &Mono) {
    //let domain = mono.root_appdomain();
    //domain.set();
    let rootdomain = mono.root_appdomain();

    unsafe {
        //let corlib = Image { image: unsafe { native::mono_get_corlib() } };
        /*
        let adclass = corlib.class_from_name(cstr!("System"), cstr!("AppDomain")).unwrap();
        let createfun = adclass.methods().find(|x| x.name().to_str().unwrap() == "CreateDomain").unwrap();
        let unloadfun = adclass.methods().find(|x| x.name().to_str().unwrap() == "Unload").unwrap();
        let createargs = ObjectArray::from_iter(rootdomain, &Class(unsafe { native::mono_get_object_class() })
, &[MonoString::new("mydomain", rootdomain)]);
        let cretin = createfun.invoke_array(ObjectReference::null(), &createargs);
        println!("cretin = {:?}", cretin);
        let cretin = cretin.unwrap();

        let dom = native::mono_domain_from_appdomain(cretin.raw() as *mut native::MonoAppDomain);
        let dom = AppDomain(dom);
         */
        /*
        {
            let image = mono.image_open().unwrap();
            let assembly = dom.load_assembly(&image).unwrap();
            println!("assembly loaded");
            std::mem::forget(assembly);

            std::mem::forget(image);
            println!("image dropped");
        }

        std::mem::forget(dom);
         */


        /*
        let unloadargs = ObjectArray::from_iter(rootdomain, &Class(unsafe { native::mono_get_object_class() }), &[cretin.deref().unwrap()]);
        let uret = unloadfun.invoke_array(ObjectReference::null(), &unloadargs);
        println!("uret = {:?}", uret);
         */



        let img = mono.open_image("/tmp/file.exe").unwrap();
        {
            let klass = img.class_from_name(cstr!("MyNS"), cstr!("Test")).unwrap();
            println!("Got class: {:?}", klass);
        }
        drop(img);
        //println!("Got class: {}", klass);


        /* THIS WORKS !!!!!!!!!!!
        let dom = mono.create_appdomain();

        dom.set();

        //let ass = native::mono_domain_assembly_open(dom.0, cstr!("/tmp/file.exe"));
        let mut status = native::MonoImageOpenStatus::MONO_IMAGE_OK;
        let img = native::mono_image_open(cstr!("/tmp/file.exe"), &mut status);
        assert!(!img.is_null());
        let ass = native::mono_assembly_load_from(img, cstr!("file"), &mut status);
        assert!(!ass.is_null());
        native::mono_image_close(img);

        rootdomain.set();

        drop(dom);
         */

        //std::mem::forget(corlib);
    }

    /*
    let image = mono.image_open().unwrap();
    let domain = mono.create_appdomain();
    println!("domain up");

    {
        let assembly = domain.load_assembly(&image).unwrap();
        println!("assembly loaded");
        drop(assembly);
    }

    //drop(image);
    //println!("image dropped");
    drop(domain);
    println!("domain dropped");*/

    //
    return;

    let image = mono.open_image("/tmp/file.exe").unwrap();
    let domain = mono.root_appdomain();

    let ass = domain.load_assembly(&image).unwrap();



    let klass = image./*get_class(TypeToken(0x2000002))*/class_from_name(cstr!("MyNS"), cstr!("Test")).unwrap();
    println!("{:?}", klass);
    for method in klass.methods() {
        println!("method: {:?} {:x}", method.name(), method.token().0);
    }

    let meth = image.get_method(MethodToken(0x6000003)).unwrap();
    println!("selected {:?}", meth.name());
//    let args = domain.new_string_array().unwrap();

    //let ao = ObjectArray::new(domain,
    //ao.set(0, args);
    let args = ObjectArray::from_iter(domain, &unsafe { Class::from_raw(native::mono_get_string_class()) }, &[MonoString::empty(domain), MonoString::new("PogChamp", domain)]);
    let ao = ObjectArray::from_iter(domain, &unsafe { Class::from_raw(native::mono_get_object_class()) }, &[args]);

    //let ret = meth.invoke(&[unsafe { GenericObject::from_ptr(args.0 as *mut native::MonoObject) }]);
    let ret = meth.invoke_array(Null, &ao);
    println!("{:?}", ret);

        /*
    let ass = domain.load_assembly(&image).unwrap();
    ass.execute(&["lul", "more", "args"]);
    println!("Hello, world!");
         */

    //mono.root_appdomain().set();
}
*/
