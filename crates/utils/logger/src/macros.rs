#[macro_export]
macro_rules! log {
    ($level:ident, $event:expr, $($k:ident = $v:expr),* $(,)?) => {{
        $crate::tracing::$level!(
            event = $event,
            $($k = %$v,)*
        );
    }};
}

#[macro_export]
macro_rules! info {
    ($event:expr $(, $k:ident = $v:expr)* $(,)?) => {
        $crate::log!(info, $event, $($k = $v),*);
    };
}

#[macro_export]
macro_rules! debug {
    ($event:expr $(, $k:ident = $v:expr)* $(,)?) => {
        $crate::log!(debug, $event, $($k = $v),*);
    };
}

#[macro_export]
macro_rules! error {
    ($event:expr $(, $k:ident = $v:expr)* $(,)?) => {
        $crate::log!(error, $event, $($k = $v),*);
    };
}

#[macro_export]
macro_rules! warn {
    ($event:expr $(, $k:ident = $v:expr)* $(,)?) => {
        $crate::log!(warn, $event, $($k = $v),*);
    };
}

#[macro_export]
macro_rules! trace {
    ($event:expr $(, $k:ident = $v:expr)* $(,)?) => {
        $crate::log!(trace, $event, $($k = $v),*);
    };
}
