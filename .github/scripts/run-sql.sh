#!/usr/bin/env bash
# Runs a SQL file from assetstore-demo (--sql-out) in the db container on the assetStore server, through AWS SSM.
# Usage: run-sql.sh <file> <label>. Needs AWS credentials (aws-actions/configure-aws-credentials) and INSTANCE_ID.
set -euo pipefail
FILE=$1
LABEL=$2
if [ ! -s "$FILE" ]; then
  echo "No SQL to run"
  exit 0
fi

SQL=$(base64 -w0 "$FILE")
COMMAND="cd /home/ubuntu/assetStore && echo $SQL | base64 -d | docker compose exec -T db psql -U assetstore -d assetstore -v ON_ERROR_STOP=1 -q"
jq -n --arg command "$COMMAND" '{commands: [$command]}' > ssm-params.json
ID=$(aws ssm send-command --instance-ids "$INSTANCE_ID" --document-name AWS-RunShellScript \
  --comment "Demo data $LABEL ($GITHUB_RUN_ID)" --parameters file://ssm-params.json --query Command.CommandId --output text)
echo "SSM command $ID"

STATUS=Pending
for _ in $(seq 1 30); do
  sleep 5
  STATUS=$(aws ssm get-command-invocation --command-id "$ID" --instance-id "$INSTANCE_ID" \
    --query Status --output text 2>/dev/null || echo Pending)
  case "$STATUS" in Pending|InProgress|Delayed) ;; *) break ;; esac
done
aws ssm get-command-invocation --command-id "$ID" --instance-id "$INSTANCE_ID" --query StandardErrorContent --output text >&2

STATEMENTS=$(grep -cE '^(UPDATE|DELETE)' "$FILE" || true)
if [ "$STATUS" = Success ]; then
  echo "> 🗄️ SQL na serveru ($LABEL): OK, $STATEMENTS příkazů" >> "$GITHUB_STEP_SUMMARY"
else
  echo "> [!CAUTION]" >> "$GITHUB_STEP_SUMMARY"
  echo "> SQL na serveru ($LABEL) selhalo: **$STATUS**. Podrobnosti v logu kroku." >> "$GITHUB_STEP_SUMMARY"
  exit 1
fi
