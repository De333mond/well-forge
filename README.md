# Well Forge

Desktop-приложение на Tauri 2 с Vue 3, TypeScript, Vite, Tailwind CSS 4 и shadcn-vue. Стартовый экран показывает вызов существующей Rust-команды приветствия.

## Локальный запуск

### Dev Container

Если системные зависимости конфликтуют с пакетами хоста, откройте проект в VS Code и выберите **Dev Containers: Reopen in Container**. Нужны Docker, расширение Dev Containers и работающий X11/XWayland-сеанс на Linux. Контейнер установит Node.js, Rust и библиотеки Tauri; `npm ci` выполнится при создании контейнера. В терминале контейнера запустите `just run` или `just build`.

Для показа окна на хосте конфигурация передаёт `DISPLAY` и файл `XAUTHORITY`, а для X11 MIT-SHM использует IPC хоста. Если `XAUTHORITY` не задан, сначала укажите путь к файлу авторизации X11 в окружении VS Code. Зависимости frontend и результат сборки Rust хранятся в отдельных Docker-томах, поэтому локальные `node_modules` и `src-tauri/target` не смешиваются с контейнерными. Dev Container рассчитан на Linux с X11/XWayland; Windows-сборка по-прежнему выполняется в GitHub Actions.

### На хосте

Для запуска без контейнера нужны Node.js, Rust и [системные зависимости Tauri](https://v2.tauri.app/start/prerequisites/) для вашей ОС.

На CachyOS/Arch установите системные зависимости Tauri:

```sh
sudo pacman -Syu --needed \
  webkit2gtk-4.1 base-devel curl wget file openssl \
  appmenu-gtk-module libappindicator librsvg xdotool
```

```sh
npm ci
just run
```

`just build` собирает desktop-приложение. Для работы только с frontend используйте `just frontend-run` или `just frontend-build`; вторая команда проверяет TypeScript и собирает файлы в `dist`. Список рецептов: `just`.

Тема переключается в верхней панели: светлая, тёмная или системная. По умолчанию используется системная; выбор сохраняется между запусками.

Основные каталоги frontend: `src/app` — запуск и маршрутизация, `src/components` — общие компоненты, `src/features` — экраны и небольшие примеры, `src/composables` — применение темы, `src/stores` — состояние, `src/lib` — интеграция с Tauri и утилиты, `src/styles` — токены и глобальные стили.

## Сборки GitHub Actions

Workflow [Build desktop app](.github/workflows/build.yml) запускается при отправке любого тега и вручную через **Actions → Build desktop app → Run workflow**. В артефактах завершённого запуска доступны:

- `well-forge-windows-x86_64` — `well-forge.exe`;
- `well-forge-linux-x86_64` — `well-forge-linux-x86_64.tar.gz` с исполняемым файлом `well-forge`.

Linux-сборке нужны установленные GTK/WebKitGTK 4.1 и связанные системные библиотеки. После распаковки Linux-архива запустите `./well-forge`.
