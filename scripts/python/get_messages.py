import requests

# Define the endpoint URL and parameters
# url = 'https://frequency-rococo-rpc.dwellir.com'
url = 'https://frequency-rpc.dwellir.com'



# Open the file for appending
with open('results_mainnet.txt', 'a') as f:
	# Loop until there are no more pages

	for schema_id in range(5, 11):
		starting_block = 1
		last_block = 2293699
		page_size = 10000
		step = 50000
		print("getting schema " + str(schema_id))

		while starting_block - step < last_block:
			if starting_block >= min(starting_block + step, last_block):
				break
			# Make the HTTP request
			params = {
				"id": 200,
				"jsonrpc": "2.0",
				"method": "messages_getBySchemaId",
				"params": [schema_id, [starting_block, 0, min(starting_block + step, last_block), page_size]]
			}
			print(params['params'])
			response = requests.post(url, json=params)

			# Check for HTTP errors
			response.raise_for_status()

			# Get the JSON data from the response
			data = response.json()

			# Check if there are any items in the data
			if not data:
				break

			print(data)
			# Append the items to the file
			res = data.get('result')
			for item in res['content']:
				f.write(str(item) + '\n')

			# Increment the page number for the next request
			starting_block += step

print('Results appended to results.txt')
