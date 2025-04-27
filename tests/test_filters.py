import os
from PIL import Image
from app import apply_filter, RESULT_FOLDER

def create_test_image(path):
    # Crée une image 3x3 px avec divers niveaux de couleurs
    img = Image.new('RGB', (3, 3), color=(200, 200, 200))
    img.putpixel((0, 0), (255, 255, 255))  # clair
    img.putpixel((1, 1), (10, 10, 10))     # foncé
    img.save(path)

def test_apply_filter_jaune(tmp_path):
    img_path = tmp_path / "img.png"
    create_test_image(str(img_path))

    result_name = apply_filter(str(img_path), "jaune", niveau_blanc=180)
    result_path = os.path.join(RESULT_FOLDER, result_name)
    assert os.path.exists(result_path)

    img = Image.open(result_path)
    # Pixel clair => jaune
    assert img.getpixel((0, 0)) == (254, 254, 1)
    # Pixel foncé => inversé
    assert img.getpixel((1, 1)) == (10, 200, 200)

def test_apply_filter_noir_blanc(tmp_path):
    img_path = tmp_path / "img2.png"
    create_test_image(str(img_path))

    result_name = apply_filter(str(img_path), "nb")
    result_path = os.path.join(RESULT_FOLDER, result_name)
    assert os.path.exists(result_path)

    img = Image.open(result_path)
    assert img.getpixel((0, 0)) == (254, 254, 254)
    assert img.getpixel((1, 1)) == (0, 0, 0)
