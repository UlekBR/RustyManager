#!/bin/bash

# script by chatgpt, rsrsrs

if [ $# -ne 1 ]; then
    echo "Uso: $0 <branch>"
    echo "Exemplo: $0 beta"
    exit 1
fi

branch=$1
if [[ "$branch" != "beta" && "$branch" != "main" ]]; then
    echo "Branch inválida. Use 'beta' ou 'main'."
    exit 1
fi

git checkout $branch
git add .

echo "Digite a mensagem do commit:"
read commit_message
git commit -m "$commit_message"

git push -u origin $branch

echo "Upload feito com sucesso para a branch $branch."
