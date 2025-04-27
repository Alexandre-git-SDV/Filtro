# Utilise une image python officielle comme parent
FROM python:3.12-slim

# Définit le répertoire de travail dans le conteneur
WORKDIR /app

# Copie les fichiers nécessaires
COPY . /app

# Installation des dépendances Python
RUN pip install --no-cache-dir -r requirements.txt

# Expose le port de Flask (par défaut 5000)
EXPOSE 5000

# Commande pour lancer l’application Flask
CMD ["python", "app.py"]
