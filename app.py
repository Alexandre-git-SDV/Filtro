from flask import Flask, render_template, request, redirect, url_for
from PIL import Image
import os

app = Flask(__name__)

UPLOAD_FOLDER = 'static/uploads/'
RESULT_FOLDER = 'static/results/'
os.makedirs(UPLOAD_FOLDER, exist_ok=True)
os.makedirs(RESULT_FOLDER, exist_ok=True)

def apply_filter(image_path, color_filter, niveau_blanc=180):  # Niveau de blanc par défaut
    img = Image.open(image_path)
    largeur_img, hauteur_img = img.size

    # --- Parcours de chaque pixel de l'image ---
    for y in range(hauteur_img):
        for x in range(largeur_img):
            r, v, b = img.getpixel((x, y))
            # --- Si le pixel est "clair" ---
            if r > niveau_blanc and v > niveau_blanc and b > niveau_blanc:
                # ---- FILTRE JAUNE ----
                if color_filter == "jaune":
                    img.putpixel((x, y), (254, 254, 1))
                # ---- FILTRE VERT ----
                elif color_filter == "vert":
                    img.putpixel((x, y), (145, 254, 1))
                # ---- FILTRE ROUGE ----
                elif color_filter == "rouge":
                    img.putpixel((x, y), (254, 1, 1))
                # ---- FILTRE NOIR ET BLANC (BLANC) ----
                elif color_filter == "nb":
                    img.putpixel((x, y), (254, 254, 254))
            else:
                # ---- FILTRE JAUNE (Foncé) ----
                if color_filter == "jaune":
                    img.putpixel((x, y), (b, v, r))
                # ---- FILTRE VERT (Foncé) ----
                elif color_filter == "vert":
                    img.putpixel((x, y), (r, b, v))
                # ---- FILTRE ROUGE (Foncé) ----
                elif color_filter == "rouge":
                    img.putpixel((x, y), (b, r, v))
                # ---- FILTRE NOIR ET BLANC (NOIR) ----
                # elif color_filter == "nb":
                #     img.putpixel((x, y), (0, 0, 0))
    
    result_filename = f"{color_filter}_{os.path.basename(image_path)}"
    result_path = os.path.join(RESULT_FOLDER, result_filename)
    img.save(result_path)
    return result_filename

@app.route('/', methods=['GET', 'POST'])
def index():
    uploaded_file = None
    filtered_image = None
    if request.method == 'POST':
        # Upload (si nouvelle image)
        if 'file' in request.files:
            file = request.files['file']
            if file.filename != '':
                file_path = os.path.join(UPLOAD_FOLDER, file.filename)
                file.save(file_path)
                uploaded_file = file.filename
        elif 'action' in request.form:
            # Application d'un filtre
            filter_color = request.form['action']
            image_name = request.form['image_name']
            file_path = os.path.join(UPLOAD_FOLDER, image_name)
            result_filename = apply_filter(file_path, filter_color)
            filtered_image = result_filename
            uploaded_file = image_name

    # Affiche la page avec l'image uploadée (si présente) et filtre choisi (si présent)
    return render_template('index.html', uploaded_file=uploaded_file, filtered_image=filtered_image)

if __name__ == '__main__':
    app.run(host="0.0.0.0", port=5000, debug=True)
    # app.run(debug=True)  # Pour le développement, pas nécessaire en production

