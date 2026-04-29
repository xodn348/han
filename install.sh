#!/bin/bash
set -e

echo "=== Han (한) 설치 ==="
echo ""

OS=$(uname -s)

# Pre-flight: cargo (rustup) 확인
echo "[1/5] cargo 확인..."
if ! command -v cargo &>/dev/null; then
    echo "  오류: cargo 명령을 찾을 수 없습니다."
    echo "  Han을 빌드하려면 Rust 툴체인(rustup)이 필요합니다."
    echo ""
    echo "  아래 명령으로 rustup을 설치한 뒤 다시 시도해 주세요:"
    echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    echo "  설치 후 새 셸을 열거나 'source \$HOME/.cargo/env' 를 실행하세요."
    exit 1
fi
echo "  cargo 확인 완료"

# Pre-flight: clang 확인 (LLVM 네이티브 컴파일에 필요)
echo "[2/5] clang 확인..."
if ! command -v clang &>/dev/null; then
    echo "  오류: clang 명령을 찾을 수 없습니다."
    echo "  Han의 네이티브 컴파일(LLVM)에는 clang이 필요합니다."
    echo ""
    if [ "$OS" = "Darwin" ]; then
        echo "  macOS에서는 다음 명령으로 설치할 수 있습니다:"
        echo "    xcode-select --install"
    elif [ "$OS" = "Linux" ]; then
        echo "  Linux에서는 배포판에 맞는 명령으로 설치하세요:"
        echo "    sudo apt install clang     # Debian/Ubuntu"
        echo "    sudo dnf install clang     # Fedora/RHEL"
    else
        echo "  사용 중인 플랫폼($OS)에 맞춰 clang을 설치한 뒤 다시 시도해 주세요."
    fi
    exit 1
fi
echo "  clang 확인 완료"

# hgl 빌드 및 설치
echo "[3/5] hgl 빌드 및 설치..."
if ! cargo install --path .; then
    echo "  오류: hgl 빌드에 실패했습니다."
    echo "  'cargo build' 출력 메시지를 확인하고 다시 시도해 주세요."
    exit 1
fi
echo "  hgl 설치 완료"

# VS Code 확장 설치 (VS Code 있으면)
if command -v code &>/dev/null; then
    echo "[4/5] VS Code 확장 설치..."
    VSIX="editors/vscode/han-language-0.1.0.vsix"
    if [ ! -f "$VSIX" ]; then
        echo "  VSIX 빌드 중..."
        cd editors/vscode
        npm install --silent 2>/dev/null
        npx @vscode/vsce package --allow-missing-repository 2>/dev/null
        cd ../..
    fi
    code --install-extension "$VSIX" --force 2>/dev/null
    echo "  VS Code 확장 설치 완료"
else
    echo "[4/5] VS Code 없음 — 확장 설치 건너뜀"
fi

# 스모크 테스트
echo "[5/5] 설치 확인..."
if hgl --version >/dev/null 2>&1; then
    hgl --version
    echo "  hgl 정상 동작 확인"
else
    echo "  경고: 'hgl' 명령을 PATH에서 찾을 수 없습니다."
    echo "  '~/.cargo/bin' 이 PATH에 포함되어 있는지 확인해 주세요. 예:"
    echo "    export PATH=\"\$HOME/.cargo/bin:\$PATH\""
    echo "  위 줄을 ~/.zshrc 또는 ~/.bashrc 에 추가하고 셸을 다시 열어 보세요."
    exit 1
fi

echo ""
echo "=== 설치 완료 ==="
echo ""
echo "사용법:"
echo "  hgl interpret hello.hgl    # 인터프리터로 실행"
echo "  hgl build hello.hgl        # 네이티브 바이너리 컴파일"
echo "  hgl repl                   # 대화형 REPL"
echo ""
echo "문서: https://xodn348.github.io/han/introduction.html"
echo "플레이그라운드: https://xodn348.github.io/han/playground/"
