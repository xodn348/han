Output assets/demo.gif

Set Shell "bash"
Set FontSize 14
Set FontFamily "Menlo"
Set Width 900
Set Height 600
Set Theme "Dracula"

Type "hgl repl"
Enter
Sleep 2s

Type '변수 과일 = ["사과", "배", "포도"]'
Enter
Sleep 1s

Type "반복 f 안에서 과일 { 출력(f) }"
Enter
Sleep 1.5s

Type '출력(과일.길이())'
Enter
Sleep 1s

Type '변수 점수 = 사전("수학", 95, "영어", 88)'
Enter
Sleep 1s

Type '출력(점수["수학"])'
Enter
Sleep 1s

Type '출력(현재시간())'
Enter
Sleep 1s

Type "나가기"
Enter
Sleep 1s
