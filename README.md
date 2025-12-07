# 🎵 Mobinogi MML 변환기

MIDI 파일을 마비노기 MML 형식으로 변환하는 Windows 데스크톱 애플리케이션입니다.

## 📥 다운로드

> **Windows 10/11 (64-bit) 전용**

### 💻 단독 실행 파일
**[📥 Mobinogi MML 변환기 다운로드 (v1.0.0)](https://drive.google.com/file/d/1anSF755NAwo1wb37pyES7KvG7kTFLII2/view?usp=sharing)**

압축 해제 후 바로 실행 가능합니다. 설치 불필요.

---

## ✨ 주요 기능

- **🎯 드래그 앤 드롭**: MIDI 파일을 끌어다 놓으면 즉시 변환
- **🎹 두 가지 변환 모드**
  - **일반 변환**: 음정별로 자동 파트 분리 (멜로디, 화음1, 화음2...)
  - **악기별 변환**: 악기별로 독립적인 파트 생성
- **⚙️ 설정 저장**: 변환 모드와 글자수 제한 설정이 자동으로 저장됩니다
- **📋 원클릭 복사**: 각 파트를 클립보드에 바로 복사
- **🎨 모던 UI**: 직관적이고 깔끔한 인터페이스

## 🚀 사용 방법

1. 프로그램 실행
2. 변환 옵션 설정 (선택사항)
   - **변환 모드**: 일반 / 악기별
   - **악보 글자 수**: 500~5000자 (기본 1200자)
3. MIDI 파일(.mid, .midi)을 창에 드래그하거나 클릭하여 선택
4. 변환 완료 후 원하는 파트의 **"MML 복사하기"** 버튼 클릭
5. 마비노기에서 Ctrl+V로 붙여넣기

## 📊 출력 형식

### 일반 변환 모드
```
멜로디 (주 멜로디 라인)
화음1, 화음2, 화음3... (동시 발음별 자동 분리)
```

### 악기별 변환 모드
```
멜로디 (Acoustic Grand Piano)
화음1 (String Ensemble-1)
화음2 (String Ensemble-2)
...
```

각 파트는 설정한 글자 수 이내로 자동 크롭되며, 모든 파트의 연주 시간이 동일하게 조정됩니다.

## 💡 팁

- **설정은 자동 저장**: 한 번 설정하면 다음 실행 시에도 유지됩니다
- **글자 수 제한**: 마비노기 악보당 글자 수 제한을 고려하여 조절하세요
- **복잡한 MIDI**: 트랙이 많은 경우 악기별 변환을 사용하면 더 정리된 결과를 얻을 수 있습니다

## 🛠️ 기술 스택

- **Backend**: Rust + Tauri v2
- **Frontend**: Svelte 5 + TypeScript
- **Styling**: Tailwind CSS + DaisyUI
- **MIDI Parser**: midly

## 🔧 개발자용

### 요구사항
- Node.js 18+
- Rust (latest stable)
- Windows 10/11

### 개발 환경 설정
```bash
# 의존성 설치
npm install

# 개발 모드 실행 (Hot Reload)
npm run tauri dev

# 프로덕션 빌드
npm run tauri build
```

빌드 결과물:
- 설치 프로그램: `src-tauri/target/release/bundle/nsis/Mobinogi MML 변환기_1.0.0_x64-setup.exe`
- 실행 파일: `src-tauri/target/release/mobinogi-mml-converter.exe`

## 📧 문의

버그 리포트, 기능 제안, 기타 문의사항은 **molla202512@gmail.com**으로 연락주세요.

## 📄 라이선스

MIT License - 자유롭게 사용, 수정, 배포 가능합니다.

---

**Made with ❤️ for Mabinogi Players**