import io
from app import app

def test_index_get():
    client = app.test_client()
    resp = client.get('/')
    assert resp.status_code == 200
    assert b"Filtro" in resp.data  # ou autre texte de ta page

def test_upload_post(monkeypatch):
    client = app.test_client()
    img_data = io.BytesIO()
    from PIL import Image
    Image.new('RGB', (2, 2), color=(250, 250, 250)).save(img_data, format='PNG')
    img_data.seek(0)

    data = {'file': (img_data, 'test.png')}
    resp = client.post('/', data=data, content_type='multipart/form-data', follow_redirects=True)
    assert resp.status_code == 200
    assert b'test.png' in resp.data  # ou selon le rendu de ta page
