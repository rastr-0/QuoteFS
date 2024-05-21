import requests
import json
from dotenv import load_dotenv
from os import getenv

def get_random_quote(category):
    load_dotenv('.env')
    API_KEY = getenv('KEY')

    api_url = 'https://api.api-ninjas.com/v1/quotes?category={}'.format(category)
    response = requests.get(api_url, headers={'X-Api-Key': API_KEY})

    if response.status_code == requests.codes.ok:
        res = json.loads(response.text.encode('utf-8'))
        return res[0]['quote']
    else:
        return "Error:"

quote = get_random_quote(category='happiness')
print(quote)
