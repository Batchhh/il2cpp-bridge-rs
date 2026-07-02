//! IL2CPP Functions and Type Definitions
use crate::logger;
use once_cell::sync::OnceCell;
use std::ffi::{c_char, c_int, c_void};
use std::mem::transmute;

macro_rules! define_il2cpp_functions {
    (
        required {
            $(
                $name:ident, $type_name:ident, $symbol:literal,
                ($($arg_name:ident : $arg_type:ty),*),
                $ret_type:ty
            );* $(;)?
        }
        optional {
            $(
                $opt_name:ident, $opt_type_name:ident, $opt_symbol:literal,
                ($($opt_arg_name:ident : $opt_arg_type:ty),*),
                $opt_ret_type:ty => $opt_fallback:expr
            );* $(;)?
        }
    ) => {
        // Typedefs (both required and optional)
        $(
            pub type $type_name = unsafe extern "C" fn($($arg_name: $arg_type),*) -> $ret_type;
        )*
        $(
            pub type $opt_type_name = unsafe extern "C" fn($($opt_arg_name: $opt_arg_type),*) -> $opt_ret_type;
        )*

        /// Structure holding function pointers to IL2CPP API functions.
        ///
        /// Required fields are always present after `load` succeeds.
        /// Optional fields may be `None` on newer Unity versions that no
        /// longer export certain symbols.
        #[derive(Clone)]
        pub struct Il2CppFunctions {
            $(pub $name: $type_name,)*
            $(pub $opt_name: Option<$opt_type_name>,)*
        }

        static FUNCTIONS: OnceCell<Il2CppFunctions> = OnceCell::new();

        /// Initializes the IL2CPP functions using the provided loader.
        ///
        /// Required symbols must all resolve or `load` returns `Err`.
        /// Optional symbols that cannot be resolved are stored as `None`;
        /// calling their wrappers runs the hand-written fallback provided for
        /// each optional entry (with a warning log) so that out-parameters are
        /// always left in a well-defined state.
        pub fn load(loader: impl Fn(&str) -> *mut c_void) -> Result<usize, Vec<&'static str>> {
            let mut missing = Vec::new();
            let mut count = 0usize;

            // Required symbols — any missing is a hard error.
            $(
                let $name = {
                    let ptr = loader($symbol);
                    if ptr.is_null() {
                        missing.push($symbol);
                        None
                    } else {
                        count += 1;
                        Some(unsafe { transmute::<*mut c_void, $type_name>(ptr) })
                    }
                };
            )*

            // Optional symbols — missing is OK, stored as None.
            $(
                let $opt_name = {
                    let ptr = loader($opt_symbol);
                    if ptr.is_null() {
                        None
                    } else {
                        count += 1;
                        Some(unsafe { transmute::<*mut c_void, $opt_type_name>(ptr) })
                    }
                };
            )*

            if !missing.is_empty() {
                return Err(missing);
            }

            let funcs = Il2CppFunctions {
                $($name: $name.expect("IL2CPP required symbol checked above"),)*
                $($opt_name: $opt_name,)*
            };
            if FUNCTIONS.set(funcs).is_err() {
                logger::warning("Il2Cpp functions already initialized!");
            }

            Ok(count)
        }

        fn get_functions() -> &'static Il2CppFunctions {
            FUNCTIONS.get().expect("Il2Cpp functions not initialized!")
        }

        // Required symbol wrappers — direct call.
        $(
            #[inline(always)]
            pub unsafe fn $name($($arg_name: $arg_type),*) -> $ret_type {
                (get_functions().$name)($($arg_name),*)
            }
        )*

        // Optional symbol wrappers — run the per-function hand-written
        // fallback when the symbol is missing, so the call's contract (e.g.
        // out-parameters being written) is still preserved.
        $(
            #[inline(always)]
            pub unsafe fn $opt_name($($opt_arg_name: $opt_arg_type),*) -> $opt_ret_type {
                match get_functions().$opt_name {
                    Some(f) => f($($opt_arg_name),*),
                    None => {
                        logger::warning(&format!(
                            "IL2CPP optional symbol not loaded: {}",
                            $opt_symbol
                        ));
                        $opt_fallback
                    }
                }
            }
        )*
    };
}

define_il2cpp_functions! {
    required {
    // Initialization and Configuration
    init, Il2CppInit, "il2cpp_init", (domain_name: *const c_char), ();
    init_utf16, Il2CppInitUtf16, "il2cpp_init_utf16", (domain_name: *const u16), ();
    shutdown, Il2CppShutdown, "il2cpp_shutdown", (), ();
    set_config_dir, Il2CppSetConfigDir, "il2cpp_set_config_dir", (config_path: *const c_char), ();
    set_data_dir, Il2CppSetDataDir, "il2cpp_set_data_dir", (data_path: *const c_char), ();
    set_temp_dir, Il2CppSetTempDir, "il2cpp_set_temp_dir", (temp_path: *const c_char), ();
    set_config, Il2CppSetConfig, "il2cpp_set_config", (executable_path: *const c_char), ();
    set_config_utf16, Il2CppSetConfigUtf16, "il2cpp_set_config_utf16", (executable_path: *const u16), ();
    set_commandline_arguments, Il2CppSetCommandlineArguments, "il2cpp_set_commandline_arguments", (argc: c_int, argv: *const *const c_char, basedir: *const c_char), ();
    set_commandline_arguments_utf16, Il2CppSetCommandlineArgumentsUtf16, "il2cpp_set_commandline_arguments_utf16", (argc: c_int, argv: *const *const u16, basedir: *const c_char), ();
    set_memory_callbacks, Il2CppSetMemoryCallbacks, "il2cpp_set_memory_callbacks", (callbacks: *mut c_void), ();

    // Memory operations
    alloc, Il2CppAlloc, "il2cpp_alloc", (size: usize), *mut c_void;
    free, Il2CppFree, "il2cpp_free", (ptr: *mut c_void), ();

    // GC operations
    gc_collect, Il2CppGcCollect, "il2cpp_gc_collect", (max_generations: c_int), ();
    gc_get_used_size, Il2CppGcGetUsedSize, "il2cpp_gc_get_used_size", (), i64;
    gc_get_heap_size, Il2CppGcGetHeapSize, "il2cpp_gc_get_heap_size", (), i64;

    // GCHandle operations
    gchandle_new, Il2CppGchandleNew, "il2cpp_gchandle_new", (obj: *mut c_void, pinned: bool), u32;
    gchandle_new_weakref, Il2CppGchandleNewWeakref, "il2cpp_gchandle_new_weakref", (obj: *mut c_void, track_resurrection: bool), u32;
    gchandle_get_target, Il2CppGchandleGetTarget, "il2cpp_gchandle_get_target", (gchandle: u32), *mut c_void;
    gchandle_free, Il2CppGchandleFree, "il2cpp_gchandle_free", (gchandle: u32), ();

    // Array operations
    array_new, Il2CppArrayNew, "il2cpp_array_new", (element_type_info: *mut c_void, length: u32), *mut c_void;
    array_class_get, Il2CppArrayClassGet, "il2cpp_array_class_get", (element_class: *mut c_void, rank: u32), *mut c_void;
    array_length, Il2CppArrayLength, "il2cpp_array_length", (array: *mut c_void), u32;
    array_get_byte_length, Il2CppArrayGetByteLength, "il2cpp_array_get_byte_length", (array: *mut c_void), u32;
    array_new_specific, Il2CppArrayNewSpecific, "il2cpp_array_new_specific", (array_type_info: *mut c_void, length: usize), *mut c_void;
    array_new_full, Il2CppArrayNewFull, "il2cpp_array_new_full", (array_class: *mut c_void, lengths: *mut usize, lower_bounds: *mut usize), *mut c_void;
    bounded_array_class_get, Il2CppBoundedArrayClassGet, "il2cpp_bounded_array_class_get", (element_class: *mut c_void, rank: u32, bounded: bool), *mut c_void;
    array_element_size, Il2CppArrayElementSize, "il2cpp_array_element_size", (array_class: *mut c_void), c_int;

    // Assembly operations
    assembly_get_image, Il2CppAssemblyGetImage, "il2cpp_assembly_get_image", (assembly: *mut c_void), *mut c_void;

    // Domain operations
    domain_get, Il2CppDomainGet, "il2cpp_domain_get", (), *mut c_void;
    domain_get_assemblies, Il2CppDomainGetAssemblies, "il2cpp_domain_get_assemblies", (domain: *mut c_void, size: *mut usize), *mut *mut c_void;
    domain_assembly_open, Il2CppDomainAssemblyOpen, "il2cpp_domain_assembly_open", (domain: *mut c_void, name: *const c_char), *mut c_void;

    // Class operations
    class_from_name, Il2CppClassFromName, "il2cpp_class_from_name", (image: *mut c_void, namespaze: *const c_char, name: *const c_char), *mut c_void;
    class_from_system_type, Il2CppClassFromSystemType, "il2cpp_class_from_system_type", (type_obj: *mut c_void), *mut c_void;
    class_from_type, Il2CppClassFromType, "il2cpp_class_from_type", (type_: *mut c_void), *mut c_void;
    class_from_il2cpp_type, Il2CppClassFromIl2CppType, "il2cpp_class_from_il2cpp_type", (type_: *mut c_void), *mut c_void;
    class_get_declaring_type, Il2CppClassGetDeclaringType, "il2cpp_class_get_declaring_type", (klass: *mut c_void), *mut c_void;
    class_get_element_class, Il2CppClassGetElementClass, "il2cpp_class_get_element_class", (klass: *mut c_void), *mut c_void;
    class_get_events, Il2CppClassGetEvents, "il2cpp_class_get_events", (klass: *mut c_void, iter: *mut *mut c_void), *mut c_void;
    class_get_field_from_name, Il2CppClassGetFieldFromName, "il2cpp_class_get_field_from_name", (klass: *mut c_void, name: *const c_char), *mut c_void;
    class_get_fields, Il2CppClassGetFields, "il2cpp_class_get_fields", (klass: *mut c_void, iter: *mut *mut c_void), *mut c_void;
    class_get_flags, Il2CppClassGetFlags, "il2cpp_class_get_flags", (klass: *mut c_void), c_int;
    class_get_rank, Il2CppClassGetRank, "il2cpp_class_get_rank", (klass: *mut c_void), c_int;
    class_get_image, Il2CppClassGetImage, "il2cpp_class_get_image", (klass: *mut c_void), *mut c_void;
    class_instance_size, Il2CppClassInstanceSize, "il2cpp_class_instance_size", (klass: *mut c_void), i32;
    class_num_fields, Il2CppClassNumFields, "il2cpp_class_num_fields", (klass: *mut c_void), usize;
    class_get_interfaces, Il2CppClassGetInterfaces, "il2cpp_class_get_interfaces", (klass: *mut c_void, iter: *mut *mut c_void), *mut c_void;
    class_get_method_from_name, Il2CppClassGetMethodFromName, "il2cpp_class_get_method_from_name", (klass: *mut c_void, name: *const c_char, args_count: c_int), *mut c_void;
    class_get_methods, Il2CppClassGetMethods, "il2cpp_class_get_methods", (klass: *mut c_void, iter: *mut *mut c_void), *mut c_void;
    class_get_name, Il2CppClassGetName, "il2cpp_class_get_name", (klass: *mut c_void), *const c_char;
    class_get_namespace, Il2CppClassGetNamespace, "il2cpp_class_get_namespace", (klass: *mut c_void), *const c_char;
    class_get_nested_types, Il2CppClassGetNestedTypes, "il2cpp_class_get_nested_types", (klass: *mut c_void, iter: *mut *mut c_void), *mut c_void;
    class_get_parent, Il2CppClassGetParent, "il2cpp_class_get_parent", (klass: *mut c_void), *mut c_void;
    class_get_properties, Il2CppClassGetProperties, "il2cpp_class_get_properties", (klass: *mut c_void, iter: *mut *mut c_void), *mut c_void;
    class_get_property_from_name, Il2CppClassGetPropertyFromName, "il2cpp_class_get_property_from_name", (klass: *mut c_void, name: *const c_char), *mut c_void;
    class_get_static_field_data, Il2CppClassGetStaticFieldData, "il2cpp_class_get_static_field_data", (klass: *mut c_void), *mut c_void;
    class_value_size, Il2CppClassValueSize, "il2cpp_class_value_size", (klass: *mut c_void, size: *mut usize), i32;
    class_array_element_size, Il2CppClassArrayElementSize, "il2cpp_class_array_element_size", (klass: *mut c_void), c_int;
    class_get_type, Il2CppClassGetType, "il2cpp_class_get_type", (klass: *mut c_void), *mut c_void;
    class_enum_basetype, Il2CppClassEnumBasetype, "il2cpp_class_enum_basetype", (klass: *mut c_void), *mut c_void;
    class_has_references, Il2CppClassHasReferences, "il2cpp_class_has_references", (klass: *mut c_void), bool;
    class_has_parent, Il2CppClassHasParent, "il2cpp_class_has_parent", (klass: *mut c_void, klassc: *mut c_void), bool;
    class_has_attribute, Il2CppClassHasAttribute, "il2cpp_class_has_attribute", (klass: *mut c_void, attr_class: *mut c_void), bool;
    runtime_class_init, Il2CppRuntimeClassInit, "il2cpp_runtime_class_init", (klass: *mut c_void), ();
    class_is_abstract, Il2CppClassIsAbstract, "il2cpp_class_is_abstract", (klass: *mut c_void), bool;
    class_is_assignable_from, Il2CppClassIsAssignableFrom, "il2cpp_class_is_assignable_from", (klass: *mut c_void, oklass: *mut c_void), bool;
    class_is_blittable, Il2CppClassIsBlittable, "il2cpp_class_is_blittable", (klass: *mut c_void), bool;
    class_is_enum, Il2CppClassIsEnum, "il2cpp_class_is_enum", (klass: *mut c_void), bool;
    class_is_generic, Il2CppClassIsGeneric, "il2cpp_class_is_generic", (klass: *mut c_void), bool;
    class_is_inflated, Il2CppClassIsInflated, "il2cpp_class_is_inflated", (klass: *mut c_void), bool;
    class_is_interface, Il2CppClassIsInterface, "il2cpp_class_is_interface", (klass: *mut c_void), bool;
    class_is_subclass_of, Il2CppClassIsSubclassOf, "il2cpp_class_is_subclass_of", (klass: *mut c_void, klassc: *mut c_void, check_interfaces: bool), bool;
    class_is_valuetype, Il2CppClassIsValueType, "il2cpp_class_is_valuetype", (klass: *mut c_void), bool;
    class_get_assemblyname, Il2CppClassGetAssemblyname, "il2cpp_class_get_assemblyname", (klass: *mut c_void), *const c_char;
    class_get_type_token, Il2CppClassGetTypeToken, "il2cpp_class_get_type_token", (klass: *mut c_void), u32;

    // Field operations
    field_get_parent, Il2CppFieldGetParent, "il2cpp_field_get_parent", (field: *mut c_void), *mut c_void;
    field_get_flags, Il2CppFieldGetFlags, "il2cpp_field_get_flags", (field: *mut c_void), c_int;
    field_get_name, Il2CppFieldGetName, "il2cpp_field_get_name", (field: *mut c_void), *const c_char;
    field_get_offset, Il2CppFieldGetOffset, "il2cpp_field_get_offset", (field: *mut c_void), i32;
    field_get_type, Il2CppFieldGetType, "il2cpp_field_get_type", (field: *mut c_void), *mut c_void;
    field_get_value_object, Il2CppFieldGetValueObject, "il2cpp_field_get_value_object", (field: *mut c_void, obj: *mut c_void), *mut c_void;
    field_static_get_value, Il2CppFieldStaticGetValue, "il2cpp_field_static_get_value", (field: *mut c_void, value: *mut c_void), ();
    field_static_set_value, Il2CppFieldStaticSetValue, "il2cpp_field_static_set_value", (field: *mut c_void, value: *mut c_void), ();
    field_has_attribute, Il2CppFieldHasAttribute, "il2cpp_field_has_attribute", (field: *mut c_void, attr_class: *mut c_void), bool;

    // Property operations
    property_get_flags, Il2CppPropertyGetFlags, "il2cpp_property_get_flags", (prop: *mut c_void), u32;
    property_get_get_method, Il2CppPropertyGetGetMethod, "il2cpp_property_get_get_method", (prop: *mut c_void), *mut c_void;
    property_get_set_method, Il2CppPropertyGetSetMethod, "il2cpp_property_get_set_method", (prop: *mut c_void), *mut c_void;
    property_get_name, Il2CppPropertyGetName, "il2cpp_property_get_name", (prop: *mut c_void), *const c_char;
    property_get_parent, Il2CppPropertyGetParent, "il2cpp_property_get_parent", (prop: *mut c_void), *mut c_void;

    // Image operations
    get_corlib, Il2CppGetCorlib, "il2cpp_get_corlib", (), *mut c_void;
    image_get_assembly, Il2CppImageGetAssembly, "il2cpp_image_get_assembly", (image: *mut c_void), *mut c_void;
    image_get_class, Il2CppImageGetClass, "il2cpp_image_get_class", (image: *mut c_void, index: u32), *mut c_void;
    image_get_class_count, Il2CppImageGetClassCount, "il2cpp_image_get_class_count", (image: *mut c_void), u32;
    image_get_name, Il2CppImageGetName, "il2cpp_image_get_name", (image: *mut c_void), *const c_char;
    image_get_filename, Il2CppImageGetFilename, "il2cpp_image_get_filename", (image: *mut c_void), *const c_char;
    image_get_entry_point, Il2CppImageGetEntryPoint, "il2cpp_image_get_entry_point", (image: *mut c_void), *mut c_void;

    // Method operations
    method_get_class, Il2CppMethodGetClass, "il2cpp_method_get_class", (method: *mut c_void), *mut c_void;
    method_get_flags, Il2CppMethodGetFlags, "il2cpp_method_get_flags", (method: *mut c_void, iflags: *mut c_void), u32;
    method_get_name, Il2CppMethodGetName, "il2cpp_method_get_name", (method: *mut c_void), *const c_char;
    method_get_object, Il2CppMethodGetObject, "il2cpp_method_get_object", (method: *mut c_void, refclass: *mut c_void), *mut c_void;
    method_get_from_reflection, Il2CppMethodGetFromReflection, "il2cpp_method_get_from_reflection", (method: *mut c_void), *mut c_void;
    method_get_param_count, Il2CppMethodGetParamCount, "il2cpp_method_get_param_count", (method: *mut c_void), u8;
    method_get_param_name, Il2CppMethodGetParamName, "il2cpp_method_get_param_name", (method: *mut c_void, param_index: u32), *const c_char;
    method_get_param, Il2CppMethodGetParam, "il2cpp_method_get_param", (method: *mut c_void, param_index: u32), *mut c_void;
    method_get_return_type, Il2CppMethodGetReturnType, "il2cpp_method_get_return_type", (method: *mut c_void), *mut c_void;
    method_get_token, Il2CppMethodGetToken, "il2cpp_method_get_token", (method: *mut c_void), u32;
    method_is_generic, Il2CppMethodIsGeneric, "il2cpp_method_is_generic", (method: *mut c_void), bool;
    method_is_inflated, Il2CppMethodIsInflated, "il2cpp_method_is_inflated", (method: *mut c_void), bool;
    method_is_instance, Il2CppMethodIsInstance, "il2cpp_method_is_instance", (method: *mut c_void), bool;
    method_has_attribute, Il2CppMethodHasAttribute, "il2cpp_method_has_attribute", (method: *mut c_void, attr_class: *mut c_void), bool;
    method_get_declaring_type, Il2CppMethodGetDeclaringType, "il2cpp_method_get_declaring_type", (method: *mut c_void), *mut c_void;

    // Object operations
    object_get_class, Il2CppObjectGetClass, "il2cpp_object_get_class", (obj: *mut c_void), *mut c_void;
    object_get_virtual_method, Il2CppObjectGetVirtualMethod, "il2cpp_object_get_virtual_method", (obj: *mut c_void, method: *mut c_void), *mut c_void;
    object_new, Il2CppObjectNew, "il2cpp_object_new", (klass: *mut c_void), *mut c_void;
    object_get_size, Il2CppObjectGetSize, "il2cpp_object_get_size", (obj: *mut c_void), u32;
    object_unbox, Il2CppObjectUnbox, "il2cpp_object_unbox", (obj: *mut c_void), *mut c_void;
    value_box, Il2CppValueBox, "il2cpp_value_box", (klass: *mut c_void, data: *mut c_void), *mut c_void;
    runtime_object_init, Il2CppRuntimeObjectInit, "il2cpp_runtime_object_init", (obj: *mut c_void), ();
    runtime_object_init_exception, Il2CppRuntimeObjectInitException, "il2cpp_runtime_object_init_exception", (obj: *mut c_void, exc: *mut c_void), ();

    // Runtime operations
    runtime_invoke, Il2CppRuntimeInvoke, "il2cpp_runtime_invoke", (method: *mut c_void, obj: *mut c_void, params: *const *mut c_void, exc: *mut *mut c_void), *mut c_void;
    runtime_invoke_convert_args, Il2CppRuntimeInvokeConvertArgs, "il2cpp_runtime_invoke_convert_args", (method: *mut c_void, obj: *mut c_void, params: *mut *mut c_void, param_count: c_int, exc: *mut *mut c_void), *mut c_void;
    runtime_unhandled_exception_policy_set, Il2CppRuntimeUnhandledExceptionPolicySet, "il2cpp_runtime_unhandled_exception_policy_set", (value: c_int), ();

    // Internal calls
    resolve_icall, Il2CppResolveICall, "il2cpp_resolve_icall", (name: *const c_char), *mut c_void;
    add_internal_call, Il2CppAddInternalCall, "il2cpp_add_internal_call", (name: *const c_char, method: *mut c_void), ();

    // Exception operations
    raise_exception, Il2CppRaiseException, "il2cpp_raise_exception", (exc: *mut c_void), ();
    exception_from_name_msg, Il2CppExceptionFromNameMsg, "il2cpp_exception_from_name_msg", (image: *mut c_void, name_space: *const c_char, name: *const c_char, msg: *const c_char), *mut c_void;
    get_exception_argument_null, Il2CppGetExceptionArgumentNull, "il2cpp_get_exception_argument_null", (arg: *const c_char), *mut c_void;
    format_exception, Il2CppFormatException, "il2cpp_format_exception", (ex: *const c_void, message: *mut c_char, message_size: c_int), ();
    format_stack_trace, Il2CppFormatStackTrace, "il2cpp_format_stack_trace", (ex: *const c_void, output: *mut c_char, output_size: c_int), ();
    unhandled_exception, Il2CppUnhandledException, "il2cpp_unhandled_exception", (exc: *mut c_void), ();

    // String operations
    string_new, Il2CppStringNew, "il2cpp_string_new", (str: *const c_char), *mut c_void;
    string_new_len, Il2CppStringNewLen, "il2cpp_string_new_len", (str: *const c_char, length: u32), *mut c_void;
    string_new_utf16, Il2CppStringNewUtf16, "il2cpp_string_new_utf16", (text: *const u16, len: i32), *mut c_void;
    string_new_wrapper, Il2CppStringNewWrapper, "il2cpp_string_new_wrapper", (str: *const c_char), *mut c_void;
    string_length, Il2CppStringLength, "il2cpp_string_length", (str: *mut c_void), i32;
    string_chars, Il2CppStringChars, "il2cpp_string_chars", (str: *mut c_void), *mut u16;
    string_intern, Il2CppStringIntern, "il2cpp_string_intern", (str: *mut c_void), *mut c_void;
    string_is_interned, Il2CppStringIsInterned, "il2cpp_string_is_interned", (str: *mut c_void), *mut c_void;

    // Thread operations
    thread_attach, Il2CppThreadAttach, "il2cpp_thread_attach", (domain: *mut c_void), *mut c_void;
    thread_detach, Il2CppThreadDetach, "il2cpp_thread_detach", (thread: *mut c_void), ();
    thread_current, Il2CppThreadCurrent, "il2cpp_thread_current", (), *mut c_void;
    is_vm_thread, Il2CppIsVmThread, "il2cpp_is_vm_thread", (thread: *mut c_void), bool;

    // Stack trace operations
    current_thread_walk_frame_stack, Il2CppCurrentThreadWalkFrameStack, "il2cpp_current_thread_walk_frame_stack", (func: *mut c_void, user_data: *mut c_void), ();
    thread_walk_frame_stack, Il2CppThreadWalkFrameStack, "il2cpp_thread_walk_frame_stack", (thread: *mut c_void, func: *mut c_void, user_data: *mut c_void), ();
    current_thread_get_top_frame, Il2CppCurrentThreadGetTopFrame, "il2cpp_current_thread_get_top_frame", (frame: *mut c_void), bool;
    thread_get_top_frame, Il2CppThreadGetTopFrame, "il2cpp_thread_get_top_frame", (thread: *mut c_void, frame: *mut c_void), bool;
    current_thread_get_frame_at, Il2CppCurrentThreadGetFrameAt, "il2cpp_current_thread_get_frame_at", (offset: i32, frame: *mut c_void), bool;
    thread_get_frame_at, Il2CppThreadGetFrameAt, "il2cpp_thread_get_frame_at", (thread: *mut c_void, offset: i32, frame: *mut c_void), bool;
    current_thread_get_stack_depth, Il2CppCurrentThreadGetStackDepth, "il2cpp_current_thread_get_stack_depth", (), i32;
    thread_get_stack_depth, Il2CppThreadGetStackDepth, "il2cpp_thread_get_stack_depth", (thread: *mut c_void), i32;

    // Type operations
    type_equals, Il2CppTypeEquals, "il2cpp_type_equals", (type1: *mut c_void, type2: *mut c_void), bool;
    type_get_name, Il2CppTypeGetName, "il2cpp_type_get_name", (type_: *mut c_void), *const c_char;
    type_get_object, Il2CppTypeGetObject, "il2cpp_type_get_object", (type_: *mut c_void), *mut c_void;
    type_get_type, Il2CppTypeGetType, "il2cpp_type_get_type", (type_: *mut c_void), c_int;
    type_get_class_or_element_class, Il2CppTypeGetClassOrElementClass, "il2cpp_type_get_class_or_element_class", (type_: *mut c_void), *mut c_void;
    type_get_attrs, Il2CppTypeGetAttrs, "il2cpp_type_get_attrs", (type_: *mut c_void), u32;
    type_get_assembly_qualified_name, Il2CppTypeGetAssemblyQualifiedName, "il2cpp_type_get_assembly_qualified_name", (type_: *mut c_void), *mut c_char;
    type_is_byref, Il2CppTypeIsByRef, "il2cpp_type_is_byref", (type_: *mut c_void), bool;

    // Monitor operations
    monitor_enter, Il2CppMonitorEnter, "il2cpp_monitor_enter", (obj: *mut c_void), ();
    monitor_try_enter, Il2CppMonitorTryEnter, "il2cpp_monitor_try_enter", (obj: *mut c_void, timeout: u32), bool;
    monitor_exit, Il2CppMonitorExit, "il2cpp_monitor_exit", (obj: *mut c_void), ();
    monitor_pulse, Il2CppMonitorPulse, "il2cpp_monitor_pulse", (obj: *mut c_void), ();
    monitor_pulse_all, Il2CppMonitorPulseAll, "il2cpp_monitor_pulse_all", (obj: *mut c_void), ();
    monitor_wait, Il2CppMonitorWait, "il2cpp_monitor_wait", (obj: *mut c_void), ();
    monitor_try_wait, Il2CppMonitorTryWait, "il2cpp_monitor_try_wait", (obj: *mut c_void, timeout: u32), bool;

    // Custom attributes
    custom_attrs_from_class, Il2CppCustomAttrsFromClass, "il2cpp_custom_attrs_from_class", (klass: *mut c_void), *mut c_void;
    custom_attrs_from_method, Il2CppCustomAttrsFromMethod, "il2cpp_custom_attrs_from_method", (method: *mut c_void), *mut c_void;
    custom_attrs_get_attr, Il2CppCustomAttrsGetAttr, "il2cpp_custom_attrs_get_attr", (ainfo: *mut c_void, attr_klass: *mut c_void), *mut c_void;
    custom_attrs_has_attr, Il2CppCustomAttrsHasAttr, "il2cpp_custom_attrs_has_attr", (ainfo: *mut c_void, attr_klass: *mut c_void), bool;
    custom_attrs_construct, Il2CppCustomAttrsConstruct, "il2cpp_custom_attrs_construct", (cinfo: *mut c_void), *mut c_void;
    custom_attrs_free, Il2CppCustomAttrsFree, "il2cpp_custom_attrs_free", (ainfo: *mut c_void), ();

    // Memory snapshot operations
    capture_memory_snapshot, Il2CppCaptureMemorySnapshot, "il2cpp_capture_memory_snapshot", (), *mut c_void;
    free_captured_memory_snapshot, Il2CppFreeCapturedMemorySnapshot, "il2cpp_free_captured_memory_snapshot", (snapshot: *mut c_void), ();
    set_find_plugin_callback, Il2CppSetFindPluginCallback, "il2cpp_set_find_plugin_callback", (method: *mut c_void), ();

    // Logging and debugging
    register_log_callback, Il2CppRegisterLogCallback, "il2cpp_register_log_callback", (method: *mut c_void), ();
    debugger_set_agent_options, Il2CppDebuggerSetAgentOptions, "il2cpp_debugger_set_agent_options", (options: *const c_char), ();
    is_debugger_attached, Il2CppIsDebuggerAttached, "il2cpp_is_debugger_attached", (), bool;
    unity_install_unitytls_interface, Il2CppUnityInstallUnitytlsInterface, "il2cpp_unity_install_unitytls_interface", (unitytls_interface_struct: *const c_void), ();

    // Profiler operations
    profiler_install, Il2CppProfilerInstall, "il2cpp_profiler_install", (prof: *mut c_void, shutdown_callback: *mut c_void), ();
    profiler_set_events, Il2CppProfilerSetEvents, "il2cpp_profiler_set_events", (events: u32), ();
    profiler_install_enter_leave, Il2CppProfilerInstallEnterLeave, "il2cpp_profiler_install_enter_leave", (enter: *mut c_void, leave: *mut c_void), ();
    profiler_install_allocation, Il2CppProfilerInstallAllocation, "il2cpp_profiler_install_allocation", (callback: *mut c_void), ();
    profiler_install_gc, Il2CppProfilerInstallGc, "il2cpp_profiler_install_gc", (callback: *mut c_void, heap_resize_callback: *mut c_void), ();

    // Stats operations
    stats_dump_to_file, Il2CppStatsDumpToFile, "il2cpp_stats_dump_to_file", (path: *const c_char), bool;
    stats_get_value, Il2CppStatsGetValue, "il2cpp_stats_get_value", (stat: u32), u64;
    }
    optional {
    // Thread operations — removed from required in Unity 6+ (il2cpp_thread_get_all_attached_threads)
    // Fallback: report zero attached threads (null array, *size = 0) so callers
    // never read an uninitialized out-parameter or a dangling array pointer.
    thread_get_all_attached_threads, Il2CppThreadGetAllAttachedThreads, "il2cpp_thread_get_all_attached_threads", (size: *mut usize), *mut *mut c_void => {
        if !size.is_null() { *size = 0; }
        std::ptr::null_mut()
    };

    // Testing operations — removed from required in Unity 6+ (il2cpp_class_get_bitmap_size, il2cpp_class_get_bitmap)
    // Fallbacks: report an empty/zero bitmap so the out-parameter is always
    // written and downstream consumers treat the class as having no GC bitmap.
    class_get_bitmap_size, Il2CppClassGetBitmapSize, "il2cpp_class_get_bitmap_size", (klass: *mut c_void), usize => 0;
    class_get_bitmap, Il2CppClassGetBitmap, "il2cpp_class_get_bitmap", (klass: *mut c_void, bitmap: *mut usize), () => {
        if !bitmap.is_null() { *bitmap = 0; }
    };
    }
}

pub const FIELD_ATTRIBUTE_FIELD_ACCESS_MASK: i32 = 0x0007;
/// Private field
pub const FIELD_ATTRIBUTE_PRIVATE: i32 = 0x0001;
/// Family and Assembly field
pub const FIELD_ATTRIBUTE_FAM_AND_ASSEM: i32 = 0x0002;
/// Assembly field
pub const FIELD_ATTRIBUTE_ASSEMBLY: i32 = 0x0003;
/// Family field
pub const FIELD_ATTRIBUTE_FAMILY: i32 = 0x0004;
/// Family or Assembly field
pub const FIELD_ATTRIBUTE_FAM_OR_ASSEM: i32 = 0x0005;
/// Public field
pub const FIELD_ATTRIBUTE_PUBLIC: i32 = 0x0006;
/// Static field
pub const FIELD_ATTRIBUTE_STATIC: i32 = 0x0010;
/// InitOnly (readonly) field
pub const FIELD_ATTRIBUTE_INIT_ONLY: i32 = 0x0020;
/// Literal (const) field
pub const FIELD_ATTRIBUTE_LITERAL: i32 = 0x0040;
/// Not serialized field
pub const FIELD_ATTRIBUTE_NOT_SERIALIZED: i32 = 0x0080;
/// Special name field
pub const FIELD_ATTRIBUTE_SPECIAL_NAME: i32 = 0x0200;
/// PInvoke implementation
pub const FIELD_ATTRIBUTE_PINVOKE_IMPL: i32 = 0x2000;
/// Method access mask
pub const METHOD_ATTRIBUTE_MEMBER_ACCESS_MASK: i32 = 0x0007;
/// Private method
pub const METHOD_ATTRIBUTE_PRIVATE: i32 = 0x0001;
/// Family and Assembly method
pub const METHOD_ATTRIBUTE_FAM_AND_ASSEM: i32 = 0x0002;
/// Assembly method
pub const METHOD_ATTRIBUTE_ASSEM: i32 = 0x0003;
/// Family method
pub const METHOD_ATTRIBUTE_FAMILY: i32 = 0x0004;
/// Family or Assembly method
pub const METHOD_ATTRIBUTE_FAM_OR_ASSEM: i32 = 0x0005;
/// Public method
pub const METHOD_ATTRIBUTE_PUBLIC: i32 = 0x0006;
/// Static method
pub const METHOD_ATTRIBUTE_STATIC: i32 = 0x0010;
/// Final (sealed) method — cannot be overridden
pub const METHOD_ATTRIBUTE_FINAL: i32 = 0x0020;
/// Virtual method
pub const METHOD_ATTRIBUTE_VIRTUAL: i32 = 0x0040;
/// Abstract method — no implementation
pub const METHOD_ATTRIBUTE_ABSTRACT: i32 = 0x0400;
/// New slot — method introduces a new vtable slot (not an override)
pub const METHOD_ATTRIBUTE_NEW_SLOT: i32 = 0x0100;
/// Public type
pub const TYPE_ATTRIBUTE_PUBLIC: i32 = 0x00000001;
/// Abstract type
pub const TYPE_ATTRIBUTE_ABSTRACT: i32 = 0x00000080;
/// Sealed type
pub const TYPE_ATTRIBUTE_SEALED: i32 = 0x00000100;
