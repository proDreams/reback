# Napkin Tools: Restore Backup Utility

Эта утилита позволяет выполнять бэкапы баз данных (PostgreSQL, MongoDB) и папок локально и на Amazon S3. Она отслеживает
срок хранения бэкапов и организует их в подкаталогах в соответствии с указанной конфигурацией. Функциональность
восстановления планируется для будущих релизов.

## Возможности

- Поддерживает бэкап PostgreSQL, MongoDB и Docker-баз данных.
- Бэкапит папки в локальные директории или на Amazon S3/Minio/другие S3-совместимые сервисы.
- Сохраняет бэкапы в соответствии с указанными сроками хранения.
- Логирует каждый шаг процесса бэкапа.
- Конфигурируется через файл `settings.json`.

## Требования

- Docker (необходим для бэкапов PostgreSQL и MongoDB в Docker)
- Доступ к S3 и учетные данные

## Установка

### Вариант 1: Скачать из релизов

Вы можете скачать последнюю версию с [страницы релизов](https://github.com/yourusername/backup-utility/releases).

1. Перейдите на [страницу релизов](https://github.com/yourusername/backup-utility/releases).
2. Скачайте подходящий бинарник для вашей операционной системы.
3. Распакуйте архив и сделайте бинарник исполнимым (если необходимо):

    ```bash
    chmod +x reback
    ```

### Вариант 2: Сборка локально

Если вы хотите собрать приложение самостоятельно, выполните следующие шаги:

1. Клонируйте репозиторий на вашу локальную машину:

    ```bash
    git clone https://github.com/yourusername/backup-utility.git
    cd backup-utility
    ```

2. Соберите приложение (необходимо установить Rust):

    ```bash
    cargo build --release
    ```

3. Сделайте бинарник исполнимым:

    ```bash
    chmod +x target/release/reback
    ```

4. Теперь вы можете запустить бинарник из папки `target/release` или переместить его в выбранную директорию.

## Конфигурация

Приложение конфигурируется через файл `settings.json`, который должен находиться в той же директории, что и исполняемый
файл (`universal-backup-restore`). Ниже приведен пример конфигурации:

1. Создайте файл `settings.json` рядом с исполняемым файлом (`universal-backup-restore`).
2. Скопируйте следующий шаблон в ваш `settings.json` и измените значения под вашу среду:

```json
{
  "s3_endpoint": "https://s3.example.com",
  "s3_region": "us-east-1",
  "s3_bucket": "my-bucket",
  "s3_access": "access-key",
  "s3_secret": "secret-key",
  "s3_path_style": "path",
  "backup_dir": "/tmp/backups",
  "elements": [
    {
      "element_title": "my_pg_db",
      "s3_folder": "postgres_backups",
      "backup_retention_days": 30,
      "s3_backup_retention_days": 90,
      "params": {
        "type": "postgresql",
        "db_host": "localhost",
        "db_port": 5432,
        "db_name": "my_database",
        "db_user": "user",
        "db_password": "password"
      }
    },
    {
      "element_title": "my_mongo_db",
      "s3_folder": "mongodb_backups",
      "backup_retention_days": 30,
      "s3_backup_retention_days": 90,
      "params": {
        "type": "mongodb",
        "db_host": "localhost",
        "db_port": 27017,
        "db_name": "my_mongo_database",
        "db_user": "mongo_user",
        "db_password": "mongo_password"
      }
    },
    {
      "element_title": "project_folder",
      "s3_folder": "folder_backups",
      "backup_retention_days": 30,
      "s3_backup_retention_days": 90,
      "params": {
        "type": "folder",
        "target_path": "/path/to/folder"
      }
    }
  ]
}
```

### Доступные типы бэкапов

- **Postgresql**: Бэкап базы данных PostgreSQL.
- **PostgresqlDocker**: Бэкап базы данных PostgreSQL из Docker контейнера.
- **Mongodb**: Бэкап базы данных MongoDB.
- **MongodbDocker**: Бэкап базы данных MongoDB из Docker контейнера.
- **Folder**: Бэкап локальной папки.

Каждый тип имеет свой набор параметров, как указано в конфигурационном файле.

## Использование

Для запуска процесса бэкапа выполните следующую команду:

```bash
./reback backup
```

Утилита:

1. Создаст бэкапы баз данных и папок локально и на S3.
2. Организует бэкапы в подкаталогах в зависимости от конфигурации.
3. Будет сохранять бэкапы в соответствии с указанными сроками хранения.

### Запуск как Cron задача

Вы можете автоматизировать процесс бэкапа, создав задачу Cron. Например, чтобы запускать бэкап каждый день в 2:00,
добавьте следующую строку в ваш файл crontab:

```bash
0 2 * * * /path/to/reback backup >> /path/to/logs/backup.log 2>&1
```

Эта задача Cron будет:

- Выполнять команду `universal-backup-restore backup` каждый день в 2:00.
- Перенаправлять как стандартный вывод, так и вывод ошибок в лог-файл (`backup.log`).

Для редактирования crontab используйте команду:

```bash
crontab -e
```

Затем добавьте строку задачи Cron. Убедитесь, что путь к бинарнику `universal-backup-restore` и лог-файлу указан
правильно.

## Сроки хранения бэкапов

Бэкапы сохраняются в соответствии с конфигурацией в файле `settings.json`. Параметр `backup_retention_days` указывает,
сколько дней бэкапы хранятся локально, а параметр `s3_backup_retention_days` — сколько дней бэкапы хранятся в S3.

## Автор

Этот проект был создан и поддерживается Иваном Ашихминым. Не стесняйтесь открывать задачи или вносить вклад в проект.

## Поддержка

Если вам нравится этот проект и вы хотите поддержать его дальнейшее развитие, рассмотрите возможность доната:

Донат через TON: `UQBU8rJEfUcBvJUbz6NbXiWxaOO_NoXHK_pXOWv7qsOBWbFp`

Ваша поддержка помогает проекту развиваться и улучшать будущие функции!
