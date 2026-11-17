#!/usr/bin/env bash
# One-time AWS setup for the Demo data workflow. Run in AWS CloudShell:
#   curl -fsSL https://raw.githubusercontent.com/Unicorn-University-Team/assetStore-demo/main/aws/setup-demo-role.sh | bash
#
# Role assetstore-demo-github: only the main branch of this repository can assume it (GitHub OIDC, no AWS keys),
# and it can only send a shell command to the assetStore server (for the SQL step). The OIDC provider already
# exists (assetStore/deploy/aws/setup-github-deploy.sh). Safe to run again.
set -euo pipefail

REGION=eu-central-1
INSTANCE_ID=i-0af9c67b1566f5c8d
# The token's "sub" with the repository's immutable ids:
#   gh api repos/Unicorn-University-Team/assetStore-demo/actions/oidc/customization/sub
REPO_SUBJECT=repo:Unicorn-University-Team@339749159/assetStore-demo@1461347721
ROLE=assetstore-demo-github

ACCOUNT=$(aws sts get-caller-identity --query Account --output text)
OIDC_ARN="arn:aws:iam::$ACCOUNT:oidc-provider/token.actions.githubusercontent.com"
aws iam get-open-id-connect-provider --open-id-connect-provider-arn "$OIDC_ARN" >/dev/null

cat > /tmp/demo-trust.json <<JSON
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Principal": { "Federated": "$OIDC_ARN" },
    "Action": "sts:AssumeRoleWithWebIdentity",
    "Condition": {
      "StringEquals": {
        "token.actions.githubusercontent.com:aud": "sts.amazonaws.com",
        "token.actions.githubusercontent.com:sub": "$REPO_SUBJECT:ref:refs/heads/main"
      }
    }
  }]
}
JSON
cat > /tmp/demo-policy.json <<JSON
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": "ssm:SendCommand",
      "Resource": [
        "arn:aws:ec2:$REGION:$ACCOUNT:instance/$INSTANCE_ID",
        "arn:aws:ssm:$REGION::document/AWS-RunShellScript"
      ]
    },
    { "Effect": "Allow", "Action": "ssm:GetCommandInvocation", "Resource": "*" }
  ]
}
JSON
if aws iam get-role --role-name "$ROLE" >/dev/null 2>&1; then
  aws iam update-assume-role-policy --role-name "$ROLE" --policy-document file:///tmp/demo-trust.json
else
  aws iam create-role --role-name "$ROLE" --assume-role-policy-document file:///tmp/demo-trust.json \
    --description "GitHub Actions of assetStore-demo: SQL step of the Demo data workflow" >/dev/null
fi
aws iam put-role-policy --role-name "$ROLE" --policy-name send-demo-sql --policy-document file:///tmp/demo-policy.json
echo "Role: arn:aws:iam::$ACCOUNT:role/$ROLE"
echo "DONE"
