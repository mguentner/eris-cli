# eris-cli

a simple command line application that uses [eris-rs](https://github.com/mguentner/eris-rs).

# Examples

```
$ eris-cli --file testfile.100mb --store ./eris-store -b 32 -e
urn:erisx2:B4BEGXI42YAMGDXNAUCB2WIIJQSCLKXJUPN2SDY4AN6NXODA4PH7WY7DHUJ2AI65QKSLXMSLQ42DY64B2ZML7I4YOKP53EOAQCODHCI4IM

$ eris-cli --file outfile --store ./eris-store -d -u urn:erisx2:B4BEGXI42YAMGDXNAUCB2WIIJQSCLKXJUPN2SDY4AN6NXODA4PH7WY7DHUJ2AI65QKSLXMSLQ42DY64B2ZML7I4YOKP53EOAQCODHCI4IM
done.

$ sha256sum testfile.100mb outfile
211939de08247929de075d79d60d1f57484306eaf5aa83d7cf28177acffb471c  testfile.100mb
211939de08247929de075d79d60d1f57484306eaf5aa83d7cf28177acffb471c  outfile
```

# Copyright & License

Copyright (c) 2022 Maximilian Güntner <code@sourcediver.org>

Licensed under the GPLv3.
