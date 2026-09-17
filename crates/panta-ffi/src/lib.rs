//! 最小 CXX 双向边界：验证 DTO、错误转换和 C++ 实现调用。

const MAX_REPEAT: u32 = 8;

// unsafe 只由 CXX 桥接宏生成（胶水 extern/函数/块）；边界安全前提由 cxx
// 运行时的类型检查与 ffi.hpp 签名一致性承担，本 crate 对外只暴露安全签名。
#[allow(unsafe_code)]
#[cxx::bridge(namespace = "panta::ffi")]
pub mod bridge {
    /// 只跨边界传递 UTF-8 文本和有界整数，不暴露 Qt/CAE 类型布局。
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FfiRequest {
        pub text: String,
        pub repeat: u32,
    }

    /// 成功响应保留机器可检查的重复次数，避免调用方解析展示文本。
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FfiResponse {
        pub value: String,
        pub repeat: u32,
    }

    unsafe extern "C++" {
        include!("panta/ffi.hpp");

        fn cpp_prefix() -> String;
    }

    extern "Rust" {
        fn process(request: &FfiRequest) -> Result<FfiResponse>;
        fn panic_probe();
    }
}

pub use bridge::{FfiRequest, FfiResponse};

fn process(request: &FfiRequest) -> Result<FfiResponse, String> {
    if request.text.is_empty() {
        return Err("ffi.empty_input".to_owned());
    }
    if request.repeat == 0 || request.repeat > MAX_REPEAT {
        return Err(format!("ffi.invalid_repeat: {}", request.repeat));
    }

    let value = format!(
        "{}{}",
        bridge::cpp_prefix(),
        request.text.repeat(request.repeat as usize)
    );
    Ok(FfiResponse {
        value,
        repeat: request.repeat,
    })
}

/// 边界验收专用：验证 Rust panic 在 CXX 胶水中被中止而非以异常穿越到 C++，
/// 与 `Result` 错误的可恢复路径区分；不承载业务功能。
fn panic_probe() {
    panic!("ffi.panic_probe");
}

#[cfg(test)]
mod tests {
    use super::{FfiRequest, FfiResponse, panic_probe, process};

    #[test]
    fn round_trip_calls_cpp_and_preserves_unicode() {
        let response = process(&FfiRequest {
            text: "界".to_owned(),
            repeat: 2,
        })
        .expect("valid request");
        assert_eq!(
            response,
            FfiResponse {
                value: "ffi:界界".to_owned(),
                repeat: 2,
            }
        );
    }

    #[test]
    fn empty_text_is_a_structured_error() {
        let error = process(&FfiRequest {
            text: String::new(),
            repeat: 1,
        })
        .expect_err("empty input must fail");
        assert_eq!(error, "ffi.empty_input");
    }

    #[test]
    fn repeat_bounds_are_rejected() {
        for repeat in [0, 9] {
            let error = process(&FfiRequest {
                text: "x".to_owned(),
                repeat,
            })
            .expect_err("invalid repeat must fail");
            assert!(error.starts_with("ffi.invalid_repeat:"));
        }
    }

    #[test]
    #[should_panic(expected = "ffi.panic_probe")]
    fn panic_probe_panics_with_boundary_code() {
        panic_probe();
    }
}
