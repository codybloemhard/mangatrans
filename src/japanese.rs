pub fn split_hirakata(string: &str) -> Vec<String>{
    let mut res = Vec::new();
    let chars: Vec<char> = string.chars().collect();
    if chars.is_empty() { return res; }
    let lm1 = chars.len() - 1;
    let mut i = 0;

    while i < lm1{
        let a = chars[i];
        let b = chars[i + 1];
        let comb = format!("{}{}", a, b);
        if let Hepburn::Roman(_) = Hepburn::from(&comb){
            res.push(comb);
            i += 1;
            continue;
        }
        if is_punctuation(a) {
            i += 1;
            continue;
        }
        let a = a.to_string();
        match Hepburn::from(&a){
            Hepburn::Fail => {},
            _ => res.push(a),
        }
        i += 1;
    }
    if i == lm1 && !is_punctuation(chars[lm1]){
        let last = chars[lm1].to_string();
        match Hepburn::from(&last){
            Hepburn::Fail => {},
            _ => res.push(last),
        }
    }
    res
}

pub fn romanize(string: &str) -> String{
    let mut res = String::new();
    let chars: Vec<char> = string.chars().collect();
    if chars.is_empty() { return res; }
    let lm1 = chars.len() - 1;
    let mut i = 0;
    let mut prev = chars[0];
    let mut tsu = false;

    fn push(res: &mut String, roman: &str, tsu: &mut bool, prev: &mut char){
        if *tsu{
            let next = roman.chars().next();
            if let Some(next) = next{
                if roman == &*prev.to_string(){
                    res.push('h');
                } else {
                    res.push_str(&next.to_string());
                }
            }
            *tsu = false;
        }
        res.push_str(roman);
        if let Some(last) = roman.chars().last(){ *prev = last; }
    }

    while i < lm1{
        let a = chars[i];
        let b = chars[i + 1];
        if a == ' '{
            if tsu { res.push('h'); }
            res.push(' ');
            tsu = false;
            i += 1;
            continue;
        }
        let comb = format!("{}{}", a, b);
        if let Hepburn::Roman(roman) = Hepburn::from(&comb){
            push(&mut res, &roman, &mut tsu, &mut prev);
            i += 2;
            continue;
        }
        let a = a.to_string();
        match Hepburn::from(&a){
            Hepburn::Roman(roman) => push(&mut res, &roman, &mut tsu, &mut prev),
            Hepburn::SmallTsu => tsu = true,
            Hepburn::Enlongate => res.push(prev),
            Hepburn::Fail => res.push_str(&a),
        }
        i += 1;
    }
    if i == lm1{
        let last = chars[lm1].to_string();
        if last == "っ"{
            res.push('h');
        } else if let Hepburn::Roman(roman) = Hepburn::from(&last){
            push(&mut res, &roman, &mut tsu, &mut prev);
        } else {
            res.push_str(&last);
        }
    }
    res
}

pub enum Hepburn{ Roman(String), SmallTsu, Enlongate, Fail }

impl Hepburn{
    fn from(string: &str) -> Self{
        let temp = match string{
            "あ" => "a", "ア" => "a", "い" => "i", "イ" => "i", "う" => "u", "ウ" => "u",
            "え" => "e", "エ" => "e", "お" => "o", "オ" => "o",
            "か" => "ka", "カ" => "ka", "き" => "ki", "キ" => "ki", "く" => "ku", "ク" => "ku",
            "け" => "ke", "ケ" => "ke", "こ" => "ko", "コ" => "ko",
            "さ" => "sa", "サ" => "sa", "し" => "shi", "シ" => "shi", "す" => "su", "ス" => "su",
            "せ" => "se", "セ" => "se", "そ" => "so", "ソ" => "so",
            "た" => "ta", "タ" => "ta", "ち" => "chi", "チ" => "chi", "つ" => "tsu", "ツ" => "tsu",
            "て" => "te", "テ" => "te", "と" => "to", "ト" => "to", "な" => "na", "ナ" => "na",
            "に" => "ni", "ニ" => "ni", "ぬ" => "nu", "ヌ" => "nu", "ね" => "ne", "ネ" => "ne",
            "の" => "no", "ノ" => "no",
            "は" => "ha", "ハ" => "ha", "ひ" => "hi", "ヒ" => "hi", "ふ" => "fu", "フ" => "fu",
            "へ" => "he", "ヘ" => "he", "ほ" => "ho", "ホ" => "ho",
            "ま" => "ma", "マ" => "ma", "み" => "mi", "ミ" => "mi", "む" => "mu", "ム" => "mu",
            "め" => "me", "メ" => "me", "も" => "mo", "モ" => "mo",
            "や" => "ya", "ヤ" => "ya", "ゆ" => "yu", "ユ" => "yu", "よ" => "yo", "ヨ" => "yo",
            "ら" => "ra", "ラ" => "ra", "り" => "ri", "リ" => "ri", "る" => "ru", "ル" => "ru",
            "れ" => "re", "レ" => "re", "ろ" => "ro", "ロ" => "ro",
            "わ" => "wa", "ワ" => "wa", "ゐ" => "i", "を" => "o", "ヲ" => "o", "ん" => "n", "ン" => "n",
            "が" => "ga", "ガ" => "ga", "ぎ" => "gi", "ギ" => "gi", "ぐ" => "gu", "グ" => "gu",
            "げ" => "ge", "ゲ" => "ge", "ご" => "go", "ゴ" => "go", "ざ" => "za", "ザ" => "za",
            "じ" => "ji", "ジ" => "ji", "ず" => "zu", "ズ" => "zu",
            "ぜ" => "ze", "ゼ" => "ze", "ぞ" => "zo", "ゾ" => "zo",
            "だ" => "da", "ダ" => "da", "ぢ" => "ji", "ヂ" => "ji", "づ" => "zu", "ヅ" => "zu",
            "で" => "de", "デ" => "de", "ど" => "do", "ド" => "do",
            "ば" => "ba", "バ" => "ba", "び" => "bi", "ビ" => "bi", "ぶ" => "bu", "ブ" => "bu",
            "べ" => "be", "ベ" => "be", "ぼ" => "bo", "ボ" => "bo",
            "ぱ" => "pa", "パ" => "pa", "ぴ" => "pi", "ピ" => "pi", "ぷ" => "pu", "プ" => "pu",
            "ぺ" => "pe", "ペ" => "pe", "ぽ" => "po", "ポ" => "po",
            "きゃ" => "kya", "キャ" => "kya", "きゅ" => "kyu", "キュ" => "kyu",
            "きょ" => "kyo", "キョ" => "kyo",
            "しゃ" => "sha", "シャ" => "sha", "しゅ" => "shu", "シュ" => "shu",
            "しょ" => "sho", "ショ" => "sho",
            "ちゃ" => "cha", "チャ" => "cha", "ちゅ" => "chu", "チュ" => "chu",
            "ちょ" => "cho", "チョ" => "cho",
            "にゃ" => "nya", "ニャ" => "nya", "にゅ" => "nyu", "ニュ" => "nyu",
            "にょ" => "nyo", "ニョ" => "nyo",
            "ひゃ" => "hya", "ヒャ" => "hya", "ひゅ" => "hyu", "ヒュ" => "hyu",
            "ひょ" => "hyo", "ヒョ" => "hyo",
            "みゃ" => "mya", "ミャ" => "mya", "みゅ" => "myu", "ミュ" => "myu",
            "みょ" => "myo", "ミョ" => "myo",
            "りゃ" => "rya", "リャ" => "rya", "りゅ" => "ryu", "リュ" => "ryu",
            "りょ" => "ryo", "リョ" => "ryo",
            "ぎゃ" => "gya", "ギャ" => "gya", "ぎゅ" => "gyu", "ギュ" => "gyu",
            "ぎょ" => "gyo", "ギョ" => "gyo",
            "じゃ" => "ja", "ジャ" => "ja", "じゅ" => "ju", "ジュ" => "ju",
            "じょ" => "jo", "ジョ" => "jo",
            "びゃ" => "bya", "ビャ" => "bya", "びゅ" => "byu", "ビュ" => "byu",
            "びょ" => "byo", "ビョ" => "byo",
            "ぴゃ" => "pya", "ピャ" => "pya", "ぴゅ" => "pyu", "ピュ" => "pyu",
            "ぴょ" => "pyo", "ピョ" => "pyo",
            "〜" => "~", "？" => "?", "！" => "!", "　" => " ", "「" => "\"", "」" => "\"",
            "。" => ".", "、" => ",",
            "っ" => "_", "ッ" => "_",
            "ー" => "-",
            _ => "",
        }.to_string();
        match temp.as_str(){
            "" => Self::Fail,
            "_" => Self::SmallTsu,
            "-" => Self::Enlongate,
            _ => Self::Roman(temp),
        }
    }
}

pub fn map_kanjis(strings: &[String], subs: &[[String; 2]]) -> Vec<String>{
    let mut replaceds = strings.to_vec();
    for [replacee, replacant] in subs{
        for (i, string) in replaceds.iter().enumerate(){
            let new = string.replacen(replacee, replacant, 1);
            if &new != string {
                replaceds[i] = new;
                break;
            }
        }
    }
    replaceds
}

pub fn could_contain_kanji(strings: &[String]) -> bool{
    for string in strings{
        for c in string.chars(){
            if could_be_kanji(c) { return true; }
        }
    }
    false
}

pub fn could_be_kanji(c: char) -> bool{
    !is_latin(c) && !is_hiragana(c) && !is_katakana(c) && !is_punctuation(c) && !is_whitespace(c)
}

pub fn is_latin(c: char) -> bool{
    "qgmlwyfubdstnriaeohzxcvjkpQGMLWYFUBDSTNRIAEOHZXCVJKP0123456789０１２３４５６７８９".contains(c)
}

pub fn is_hiragana(c: char) -> bool{
    "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんがぎぐげござじずぜぞだぢづでどばびぶべぼぱぴぷぺぽゐゃゅょっ".contains(c)
}

pub fn is_katakana(c: char) -> bool{
    "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲンガギグゲゴザジズゼゾダヂヅデドバビブベボパピプペポャュョッ".contains(c)
}

pub fn is_punctuation(c: char) -> bool{
    "\"'-_=+`~,./<>?\\|[]{}!@#$%^&*()〜ー！？・「」、，。".contains(c)
}

pub fn is_whitespace(c: char) -> bool{
    " 　\t\n".contains(c)
}

pub fn to_mora(c: char) -> usize{
    if "ゃゅょャュョ 　〜！？・「」、。-_=+`~,./<>?\\|[]{}!@#$%^&*(\"'".contains(c) { return 0; }
    if is_latin(c) { return 0; }
    if is_whitespace(c) { return 0; }
    1
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_romanize(){
        assert_eq!(
            &romanize("ちょうしょく は じぶんで!! つくって ください!!"),
            "choushoku ha jibunde!! tsukutte kudasai!!"
        );
        assert_eq!(
            &romanize("いってキーまーす!!"),
            "ittekiimaasu!!"
        );
        assert_eq!(
            &romanize("あっ"),
            "ah"
        );
        assert_eq!(
            &romanize("あっああ"),
            "ahaa"
        );
        assert_eq!(
            &romanize("あっ みなかみ さん？"),
            "ah minakami san?"
        );
    }

    #[test]
    fn test_map_kanjis(){
        // normal one of each
        let map = vec!
            [
                ("今", "きょ"), ("日", "う"), ("雨", "あめ"), ("降", "ふ"),
                ("作", "つく"), ("下", "くだ")
            ]
            .into_iter().map(|(a, b)| [a.to_string(), b.to_string()]).collect::<Vec<_>>();
        assert_eq!(
            map_kanjis(&[
                "今日雨降るって".to_string(),
                "作って 下さい!!".to_string()
            ], &map),
            vec![
                "きょうあめふるって".to_string(),
                "つくって ください!!".to_string()
            ]
        );
        // more than kanji mapping in one map entry
        let map = vec!
            [
                ("今日", "きょう"), ("雨", "あめ"), ("降", "ふ"), ("作", "つく"), ("下", "くだ")
            ]
            .into_iter().map(|(a, b)| [a.to_string(), b.to_string()]).collect::<Vec<_>>();
        assert_eq!(
            map_kanjis(&[
                "今日雨降るって".to_string(),
                "作って 下さい!!".to_string()
            ], &map),
            vec![
                "きょうあめふるって".to_string(),
                "つくって ください!!".to_string()
            ]
        );
        // two of the same in one line
        let map = vec!
            [
                ("考", "かんが"), ("不", "ふ"), ("幸", "こう"), ("中", "ちゅ"), ("幸", "さいわ"),
            ]
            .into_iter().map(|(a, b)| [a.to_string(), b.to_string()]).collect::<Vec<_>>();
        assert_eq!(
            map_kanjis(&[
                "そう考えると".to_string(),
                "不幸中の幸いって".to_string(),
                "ヤツだね".to_string()
            ], &map),
            vec![
                "そうかんがえると".to_string(),
                "ふこうちゅのさいわいって".to_string(),
                "ヤツだね".to_string()
            ]
        );
        // two of the same in two lines
        let map = vec!
            [
                ("考", "かんが"), ("不", "ふ"), ("幸", "こう"), ("中", "ちゅ"), ("幸", "さいわ"),
            ]
            .into_iter().map(|(a, b)| [a.to_string(), b.to_string()]).collect::<Vec<_>>();
        assert_eq!(
            map_kanjis(&[
                "そう考えると".to_string(),
                "不幸中の".to_string(),
                "幸いって".to_string(),
                "ヤツだね".to_string()
            ], &map),
            vec![
                "そうかんがえると".to_string(),
                "ふこうちゅの".to_string(),
                "さいわいって".to_string(),
                "ヤツだね".to_string()
            ]
        );
        // all of it
        let map = vec!
            [
                ("今日", "きょう"), ("考", "かんが"), ("不", "ふ"),
                ("幸", "こう"), ("中", "ちゅ"), ("幸", "さいわ"), ("幸", "justatest")
            ]
            .into_iter().map(|(a, b)| [a.to_string(), b.to_string()]).collect::<Vec<_>>();
        assert_eq!(
            map_kanjis(&[
                "今日考".to_string(),
                "不".to_string(),
                "幸幸中".to_string(),
                "幸".to_string()
            ], &map),
            vec![
                "きょうかんが".to_string(),
                "ふ".to_string(),
                "こうさいわちゅ".to_string(),
                "justatest".to_string()
            ]
        );
    }

    #[test]
    fn to_mora_test(){
        fn morae(string: &str) -> usize{
            string.chars().fold(0, |a, c| a + to_mora(c))
        }
        assert_eq!(morae("きょう"), 2);
        assert_eq!(morae("  いってキーまーす！"), 8);
    }

    #[test]
    fn split_hirakata_test(){
        let res = split_hirakata("きょう||  いってキーまーす！||つくって ください");
        assert_eq!(&res[0], "きょ");
        assert_eq!(&res[1], "う");
        assert_eq!(&res[2], "い");
        assert_eq!(&res[3], "っ");
        assert_eq!(&res[4], "て");
        assert_eq!(&res[5], "キ");
        assert_eq!(&res[6], "ま");
        assert_eq!(&res[7], "す");
        assert_eq!(&res[8], "つ");
        assert_eq!(&res[9], "く");
        assert_eq!(&res[10], "っ");
        assert_eq!(&res[11], "て");
        assert_eq!(&res[12], "く");
        assert_eq!(&res[13], "だ");
        assert_eq!(&res[14], "さ");
        assert_eq!(&res[15], "い");
    }

    #[test]
    fn could_contain_kanji_test(){
        let a = ["不幸中の幸いって".to_string()];
        let b = ["そうかんがえると".to_string()];
        assert!(could_contain_kanji(&a));
        assert!(!could_contain_kanji(&b));
    }
}
