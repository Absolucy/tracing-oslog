# tracing_oslog

This is a [tracing](https://crates.io/crates/tracing) layer for the [Apple OS logging framework](https://developer.apple.com/documentation/os/logging).

[Activities](https://developer.apple.com/documentation/os/logging/collecting_log_messages_in_activities) are used to handle spans,

## Example

```rust
use tracing_oslog::OsLogger;

let collector = tracing_subscriber::registry()
	.with(OsLogger::new("moe.absolucy.test", "default"));
tracing::subscriber::set_global_default(collector).expect("failed to set global subscriber");

let number_of_yaks = 3;
// this creates a new event, outside of any spans.
info!(number_of_yaks, "preparing to shave yaks");

let number_shaved = yak_shave::shave_all(number_of_yaks);
info!(
	all_yaks_shaved = number_shaved == number_of_yaks,
	"yak shaving completed."
);
```


## License

Copyright (c) 2021 Lucy <lucy@absolucy.moe>

This software is provided 'as-is', without any express or implied warranty. In
no event will the authors be held liable for any damages arising from the use of
this software.

Permission is granted to anyone to use this software for any purpose, including
commercial applications, and to alter it and redistribute it freely, subject to
the following restrictions:

1.  The origin of this software must not be misrepresented; you must not claim
    that you wrote the original software. If you use this software in a product,
    an acknowledgment in the product documentation would be appreciated but is
    not required.

2.  Altered source versions must be plainly marked as such, and must not be
    misrepresented as being the original software.

3.  This notice may not be removed or altered from any source distribution.

### Amendment

I, @Absolucy, fully give permission for any of my code (including the entirety of this project, tracing-oslog), anywhere, no matter the license, to be used to train machine learning models intended to be used for general-purpose programming or code analysis.
