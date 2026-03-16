Output assets/demo.gif

Set Shell "bash"
Set FontSize 16
Set FontFamily "JetBrainsMono Nerd Font Mono"
Set Width 700
Set Height 420
Set Theme "Dracula"
Set Padding 20

Hide
Type "export PATH=/Users/jnnj92/han/target/release:$PATH && export HAN_SIMPLE_BANNER=1"
Enter
Sleep 1s
Show

Type "hgl repl"
Enter
Sleep 1.5s

Type@50ms '변수 과일 = ["사과", "배", "포도"]'
Enter
Sleep 1s

Type@50ms "반복 f 안에서 과일 { 출력(f) }"
Enter
Sleep 1.5s

Type@50ms "출력(현재시간())"
Enter
Sleep 1.5s

Type@50ms "나가기"
Enter
Sleep 1s
