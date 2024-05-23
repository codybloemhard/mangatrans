# Mangatrans

Manga transcription data format and ways to render them into readable formats, statistics and more.

## Goals

The goal of this project is to be able to write manga transcriptions as data and not at the final result.
The program should parse this data and construct various forms of output from this,
such as readables, language reports and statistics.

### Transcription

A transcription should have the literal text from the manga in it.
Along with it, it could contain extra auxiliary information such as the text with the kanjis replaced,
romanized text and translation or notes.

The goal of this is to be able to keep it along while reading the manga or be able to go back and
prevent redoing work such as figuring out the spelling of kanji.

### Language Report

An extra tool for learning Japanese while reading and transcribing manga.
Could contain lists such common kanji and their mappings into hiragana.
The introduction of simple frequency statistics and sorted output may reveal patterns in the language
used such as the most common words and kanij, useful for learning.

### Statistics Report

A tool that is of novel interest. Knowing what locations show up most in the manga,
who speaks most, which characters converse lots which each other and other novelties may be
interesting to know for fans of the manga.
From my experience, as you go through a chapter to transcribe it, transcribing the language and
translating it, understanding it, dealing with kanji is much more effort than simply stating the
location and which characters are present.
That's why you might as well transcribe these things as they don't inflate the effort by much and
give rise to the opportunity to compute insights that are just really fun.

### Configuration

Transcribing into a data format and not into a format that is to be consumed directly has some
advantages.
Configuration may allow different output to be generated from the data.
The reason could be different people's preferences or different goals.
If you have done much work and realize you want different formatting, instead of reformatting your
work you can just generate a different output.
This is one of the main goals of the project since I just kept changing my formatting in the
beginning of my transcribing journey.

## Feature list

- [x] parse data
    - [x] pictures data: location, characters, nr, page
    - [x] transcription
    - [x] translation
    - [x] kanji map
    - [x] from/to (directed from to which character)
    - [x] One or more pattern
- [x] general
    - [x] report: volumes and chapters included, picture and morae counts
    - [ ] customization with config file
    - [ ] recursive locations
- [x] transcribe
    - [x] original text and translation
    - [x] kanji replacement
    - [x] automatic romanization
    - [x] automatic indentation
    - [x] text consistency improvements
    - [x] page headers
- [x] statistics
    - [x] ranked locations: appreanances, morae spoken in
    - [x] characters ranked on number of appearances
    - [x] characters ranked on morae spoken
    - [x] characters ranked on morae spoken to by other characters
    - [x] character pairs ranked on number of morae spoken in their interactions
    - [x] characters ranked on overall prominence
- [x] language report
    - [x] hiragana/katakana characters ranked by count
    - [x] kanji's ranked by count
    - [ ] words ranked by count

## Data format

The data is written down in a toml file.<br/>
Every chapter should be in it's own toml file.<br/>
Every file starts with some information about the chapter:

```toml
# this is a comment and will be ignored by the program
manga = "日常"
author = "あらゐけいいち"
volume = 1
chapter = 1
subchapter = 0.5
title = "日常の1.5"
```

The `subchapter` field is optional, the rest is expected to be there.<br/>
After that you supply an array of pictures like this:

```toml
[[pic]]
    # picture data
[[pic]]
    # picture data
[[pic]]
    # picture data
```

An example of picture data:

```toml
[[pic]]
nr = 2
page = 1
location = "shinonome house"
characters = ["nano"]
    [[pic.text]]
        # text data
    [[pic.text]]
        # text data
    [[pic.text]]
        # text data
```

Picture number(`nr`) is expected to be there.<br/>
`page` is optional and sets the page number.
If `page` is not present, it is assumed we're still on the same page
as last time you declared a page number.
Every chapter must have it's first picture assigned a page number so it's knows how to continue.<br/>
`location`, `characters` and array of texts are optional.<br/>
Every chapter's first page must have an initial location.
An example of text data:

```toml
[[pic.text]]
from = "nano"
to = "hakase"
lines = ["今日", "日直 でしたー"]
kmap = [
    ["今日", "きょう"],
    ["日", "につ"],
    ["直", "ちょく"],
]
transl = ["Today", "is my shift!"]
notes = ["This in an optional note, the transcriber may want to say something about this text."]
todo = true
```

`from` is used to describe which characters says the current text.
I personally regard the narator as a "character" in this field.<br/>
`to` holds the character to which this text is directed.
I personally regard the audience as a "character" in this field.
The field is optional. If a character speaks to themselves,
or thinks internally you can just leave this field out.<br/>
`lines` is a mandatory field and should be an array of strings,
each containing a line of transcribed text.
This text should be transcribed literally from the manga, apart from spaces
and is used for many calculations and transformations.
To split of words you can insert ascii spaces `" "` between characters.
These are removed in the output of the transcription
but will be used for things such as the romanized version.<br/>
`kmap` is an optional field that defines a mapping of kanji to hiragana or katakana.
This is used for the substitution.<br/>
`transl` is for translation of `lines`.
It's optional and you may have a different number of entries in the `transl` array than you have
in the `lines` array.<br/>
`notes` is optional and you may write down notes about this text here.
`todo` is optional and when it's set so true, it will be logged that this text needs work.
You may want to set `todo = true` if there is an error, something is incomplete etc.
This way it will be logged every time so you won't forget it after a while and you know what items
need some work.
Setting `todo = false` doesn't do anything, it's the same as leaving `todo` out.

When a kanji appears multiple times in a text, you must give the correct mapping as many
times as it appears.
This is so that it is possible to have sentences that have the same kanji with different mappings.
An example of this:

```toml
[[pic.text]]
from = "yukko"
lines = ["そう考 えると", "不幸中 の 幸いって", "ヤツだね"]
kmap = [
    ["考", "かんが"],
    ["不", "ふ"],
    ["幸", "こう"],
    ["中", "ちゅ"],
    ["幸", "さいわ"],
]
transl = ["If you think about it that way it's a blessing in disguise."]
```

### One Or More pattern

The 'One Or More' pattern has been implemented with backwards compatibility.
This means that in fields that can take multiple values with arrays,
you can now also leave out the array if you just have one value.
For example, when denoting which characters appear in a picture you could write
`characters = ["yukko", "mai"]`. For a single character you can write `characters = ["nano"]`.
With the 'One Or More' pattern you can write the former as `characters = "nano"`.
Fields that support the pattern are: `characters`, `form`, `to`, `lines`, `kmap`, `transl`
and `notes`.

### Example

For an example check out `example.toml`.
For more information on the toml language visit [https://toml.io/en/](https://toml.io/en/)

All example material such as example lines and characters in `example.toml` and this readme are
from the manga `日常` by `あらゐけいいち`.
The material is used for educational purposes.

## Usage

Program is used through a command line interface (CLI).

```
USAGE:
    mangatrans [OPTIONS] <INPUTFILES>...

ARGS:
    <INPUTFILES>...

OPTIONS:
    -d, --outputdir <OUTPUTDIR>
    -h, --help                       Print help information
    -l, --log <log>                  [default: true]
    -m, --mode <MODE>                [default: transcribe] [possible values: transcribe, stats,
                                     language]
    -o, --outputmode <OUTPUTMODE>    [default: stdout] [possible values: stdout, file]
    -V, --version                    Print version information
```

## Sample output

Sample output generated from chapter 1 of the manga 日常.

### Partial sample output of the transcription mode

- picture 65
  - text 1
    - なのちゃん <br/> 休みなんて <br/> めずらしいね
    - なのちゃん <br/> やすみなんて <br/> めずらしいね
    - nano chan <br/> yasuminante <br/> mezurashii ne
    - It's rare for Nano chan not to be present, right?
  - text 2
    - 故障かな？
    - こしょうかな?
    - koshou kana?
    - Malfunction maybe?
  - text 3
    - ちょっとやめなよー <br/> 本人バレてないと <br/> 思ってるんだから
    - ちょっとやめなよー <br/> ほんにんバレてないと <br/> おもってるんだから
    - chotto yamenayo- <br/> honnin baretenaito <br/> omotterundakara
    - Stop it, she doesn't know we found out yet.

### Sample output of the statistics mode

```
Manga: 日常
Volumes: 1
Chapters: 1
Pictures: 74
Morae spoken: 880
Locations:
    street: 30 appearances, 339 morae spoken in.
    roof: 19 appearances, 130 morae spoken in.
    classroom: 17 appearances, 311 morae spoken in.
    shinonome house: 3 appearances, 73 morae spoken in.
    chimney: 2 appearances, 7 morae spoken in.
    school hallway: 1 appearances, 12 morae spoken in.
    school grounds: 1 appearances, 8 morae spoken in.
    aioi lookalike family garden: 1 appearances, 0 morae spoken in.
Character appearances:
    yukko: 33
    nano: 24
    mio: 19
    mai: 9
    kokeshi: 6
    person: 5
    akabeko: 5
    headphones guy: 4
    aioi mom lookalike: 2
    nakanojo: 2
    izumi: 1
    crow wug: 1
    hakase: 1
    mono: 1
    chissan lookalike: 1
Morae spoken:
    yukko: 309
    nano: 238
    mio: 143
    narator: 74
    izumi: 65
    headphones guy: 16
    mai: 14
    person: 11
    nakanojo: 9
    hakase: 1
Morae spoken to:
    audience: 74
    yukko: 73
    class: 65
    mio: 55
    hakase: 48
    mai: 42
    person: 17
    nakanojo: 3
Conversation pairs in morae:
    mio, yukko: 114
    audience, narator: 74
    class, izumi: 65
    hakase, nano: 48
    mai, yukko: 40
    mai, mio: 16
    nakanojo, person: 12
    person, person: 8
```

### Partial sample output of the language mode

```
Kanji frequencies:
    降: ふ: 5
    生: せい: 4
    高: こう: 4
    子: こ: 3
    麻: ま: 3
    今日: きょう: 3
    赤: あか: 3
    衣: い: 3
    雲: のめ: 2
    行: い: 2
    先: せん: 2
    下: くだ: 2
    思: おも: 2
    日: につ: 2
    本: ほん: 2
    直: ちょく: 2
    人: にん: 2
    東: しの: 2
    当: あ: 2
    原: はら: 2
```

## License

```
Copyright (C) 2024 Cody Bloemhard

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```
