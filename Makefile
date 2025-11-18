# Nome do arquivo de chave privada de saída
OUTPUT_KEY_FILE = ed25519_key.pem
# Nome do arquivo de chave pública de saída
OUTPUT_PUB_FILE = ed25519_public.pem

# Algoritmo de criptografia
ALGORITHM = ed25519

## 🔑 Objetivos Principais
.PHONY: all
all: $(OUTPUT_PUB_FILE)
	@echo "---"
	@echo "Processo de geração concluído:"
	@echo "Chave Privada salva em: $(OUTPUT_KEY_FILE)"
	@echo "Chave Pública salva em: $(OUTPUT_PUB_FILE)"

## 🔐 Geração da Chave Privada
# Regra para gerar a chave privada (que é o requisito para a chave pública)
$(OUTPUT_KEY_FILE):
	@echo "Gerando chave privada $(ALGORITHM)..."
	@openssl genpkey -algorithm $(ALGORITHM) -out $@
	@echo "Geração da chave privada concluída."

## 🔓 Geração da Chave Pública
# Regra para extrair a chave pública da chave privada
$(OUTPUT_PUB_FILE): $(OUTPUT_KEY_FILE)
	@echo "Extraindo chave pública de $(OUTPUT_KEY_FILE)..."
	@openssl pkey -in $(OUTPUT_KEY_FILE) -pubout -out $@
	@echo "Chave pública extraída e salva."
