#!/bin/sh
curl -s -L https://en.wikipedia.org/wiki/Special:Random -w '%{stderr} %{url_effective}\n' | ./target/release/examples/content | booky kind -u
