import requests
import json 
import asyncio
import aiohttp

BACKENDURL = "http://127.0.0.1:5000"

def new_game():
	response = requests.get(f"{BACKENDURL}/new_game/")
	return json.loads(response.text)["game_id"], json.loads(response.text)["player_id"]
	 
def join_game(game_id):
	response = requests.get(f"{BACKENDURL}/join_game/{game_id}")
	return json.loads(response.text)["player_id"]

def position(game_id, player_id, position):
	data = {
		"player_id": player_id, 
		"position" : position, 
	}
	
	response = requests.post(f"{BACKENDURL}/position/{game_id}", json=data)
	return response.status_code

def shot(game_id, player_a_id, coord):
	data  = { 
		"player_id": player_a_id, 
		"x": coord[0],
		"y": coord[1],
	}
	response = requests.post(f"{BACKENDURL}/shot/{game_id}", json=data)
	print(response.text)
	return response.status_code



def strange_behaivior1(): 
	game_id, player_a_id = new_game()
	player_b_id = join_game(game_id)
	player_b_id = join_game(game_id)

def strange_behaivior2(): 
	position = [
		[[1,2], [1,3], [1,4], [1,5], [1,6]],
		[[3,2], [3,3], [3,4], [3,5]],
		[[5,2], [5,3], [5,4]],
		[[7,2], [7,3], [7,4]],
		[[9,2], [9,3]],
	]

	game_id, player_a_id = new_game()
	player_b_id = join_game(game_id)

	position(game_id, player_a_id, position)
	position(game_id, player_a_id, position)



def position_check():
	p = [
		[[1,2], [1,3], [1,4], [1,5], [1,6]],
		[[3,2], [3,3], [3,4], [3,5]],
		[[5,2], [5,3], [5,4]],
		[[7,2], [7,3], [7,4]],
		[[9,2], [9,3]],
	] #correct position

	p = [
		[["a",2], [1,3], [1,4], [1,5], [1,6]],
		[[3,2], [3,3], [3,4], [3,5]],
		[[5,2], [5,3], [5,4]],
		[[7,2], [7,3], [7,4]],
		[[9,2], [9,3]],
	] #wrong type

	p = [
		[[1,2], [1,3], [1,4], [1,5], [1,6]],
		[[2,2], [2,3], [2,4], [2,5]],
		[[3,2], [3,3], [3,4]],
		[[4,2], [4,3], [4,4]],
		[[5,2], [5,3]],
	] # errror ships touch each other 

	p = [
		[[1,2], [1,3], [1,4], [1,5], [1,6]],
		[[2,2], [2,3], [2,4], [2,5]],
		[[3,2], [3,3], [3,4]],
		[[4,2], [4,3], [4,4]],
	] # not enough numbers of ships 

	p = [
		[[1,2], [1,3], [1,4], [1,5], [1,6]],
		[[3,2], [3,3], [3,4], [3,5]],
		[[5,2], [5,3], [5,4]],
		[[7,2], [7,3]],
		[[9,2], [9,3]],
	] # wrong len of ship, not 5, 4, 3, 3, 2

	p = [
		[[1,2], [1,3], [1,4], [1,5], [1,6]],
		[[3,2], [3,3], [3,4], [3,5]],
		[[5,2], [5,3], [5,4]],
		[[7,2], [7,3], [7,4]],
		[[11,2], [11,3]],
	] # ship is outside of 10 x 10 field 

	negative_position = [
		[[1,2], [1,3], [1,4], [1,5], [1,6]],
		[[3,2], [3,3], [3,4], [3,5]],
		[[5,2], [5,3], [5,4]],
		[[7,2], [7,3], [7,4]],
		[[-2,2], [-2,3]],
	] # negative coordinates 

	p = [
		[[1,2], [1,3], [1,5], [1,6], [1,7]],
		[[3,2], [3,3], [3,4], [3,5]],
		[[5,2], [5,3], [5,4]],
		[[7,2], [7,3], [7,4]],
		[[9,2], [9,3]],
	] #gap inside a ship 

	p = [
		[[1,2], [1,3], [1,4], [1,5], [1,6]],
		[[3,2], [3,3], [3,4], [3,5]],
		[[5,2], [5,3], [5,4]],
		[[7,2], [7,3], [7,4]],
		[[7,4], [8,4]],
	] #ships overlap 












def game1(): 
	p = [
		[[1,2], [1,3], [1,4], [1,5], [1,6]],
		[[3,2], [3,3], [3,4], [3,5]],
		[[5,2], [5,3], [5,4]],
		[[7,2], [7,3], [7,4]],
		[[9,2], [9,3]],
	]

	game_id, player_a_id = new_game()
	player_b_id = join_game(game_id)
	print(f'game_id = {game_id}, player_a = {player_a_id}, player_b = {player_b_id}')


	position(game_id, player_a_id, p)
	position(game_id, player_b_id, p)
	
	print(shot(game_id, player_b_id, [1,1]))
	print(shot(game_id, player_a_id, [9,2]))
	input("waiting for first shots, any key to continue")

	print(shot(game_id, player_b_id, [1,2]))
	print(shot(game_id, player_a_id, [9,3]))
	print(shot(game_id, player_b_id, [10,10]))



	

if __name__ == "__main__":
	game1()

