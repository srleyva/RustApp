import requests
import json

existing_entries = None
with open('./seed/seed-data.txt', 'r') as f:
    entries = f.read()
    existing_entries = json.loads(entries)

print("Old: ", len(existing_entries))

new_list = existing_entries
for i in range(100):
    data = requests.get('https://randomapi.com/api/3sht6nbf', params={
        "key": "YP0V-BDGN-CFAB-IWNP",
        "results": 2500,
    }).json()

    new_data = data['results']
    print(len(new_data))

    new_list = new_data + new_list
    print("new len: ", len(new_list))

with open('./seed/seed-data.txt', 'w+') as f:
    f.write(json.dumps(new_list))