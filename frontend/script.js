document.addEventListener('DOMContentLoaded', () => {
	const OFFSET = 568
	const OFFSET_X = 25
	const OFFSET_Y = 8
	const BACKEND_URL = 'battleship.jackenbaer.com:5000'
	const FRONTEND_URL = 'battleship.jackenbaer.com:80'





	class AudioManager {
		constructor() {
			this.audioQueue = []
			this.isPlaying = false
			this.sounds = {
				newGame: '/sounds/new_game.mp3',
				shot: 'sounds/shot_1.mp3',
				hit: 'sounds/shot_2_hit.mp3',
				miss: 'sounds/shot_2_water.mp3',
				enemyHit: 'sounds/shot_3_enemy_ship_hit.mp3',
				ownHit: 'sounds/shot_3_own_ship_has_been_hit.mp3',
				lostOwnShip: 'sounds/lost_own_ship_1.mp3',
				sinking: 'sounds/enemy_ship_1_anime_ship_is_sinking.mp3',
				enemySunk: 'sounds/enemy_ship_3_anime_ship_sunk.mp3',
				ownSunk: 'sounds/lost_own_ship_3_our_ship_has_sunk.mp3',
				lostPoint: 'sounds/enemy_ship_2_lost_peep_for_each_length_point.mp3',
				ending: 'sounds/ending.mp3',
				error: 'sounds/error.mp3'
			}
		}

		addToQueue(filePath) {
			this.audioQueue.push(filePath)
			if (!this.isPlaying) {
				this.playNextInQueue()
			}
		}

		playNextInQueue() {
			if (this.audioQueue.length === 0) {
				this.isPlaying = false
				return
			}

			this.isPlaying = true
			const filePath = this.audioQueue.shift()
			const audio = new Audio(filePath)
			audio.onended = () => this.playNextInQueue()
			audio.play()
		}
	}

	class Ship {
		constructor(length, htmlObject) {
			this.length = length
			this.htmlObject = document.querySelector(htmlObject)
			this.rotation = 0
			// Bind the 'rotate' method to the current Ship instance
			this.rotate = this.rotate.bind(this)
			this.htmlObject.addEventListener('dblclick', this.rotate)

			this.handleMouseDown = this.handleMouseDown.bind(this)
			this.handleMouseMove = this.handleMouseMove.bind(this)
			this.handleMouseUp = this.handleMouseUp.bind(this)

			this.htmlObject.addEventListener('mousedown', this.handleMouseDown)
		}

		getCoordinates() {
			return this.htmlObject.coord
		}

		getLength() {
			return this.length
		}

		getRotation() {
			return this.rotation
		}

		rotationOffsetLeft() {
			const rectRect = this.htmlObject.getBoundingClientRect()
			if (this.rotation == 0) {
				return 0
			} else if (this.rotation == 90) {
				return rectRect.width / 2 - OFFSET_X
			} else {
				throw new Error('Invalid rotation value: Must be either 0 or 90')
			}
		}

		rotationOffsetTop() {
			const rectRect = this.htmlObject.getBoundingClientRect()
			if (this.rotation == 0) {
				return 0
			} else if (this.rotation == 90) {
				return -rectRect.width / 2 + OFFSET_X
			} else {
				throw new Error('Invalid rotation value: Must be either 0 or 90')
			}
		}

		rotate(event) {
			event.preventDefault() // Prevents unwanted side effects (e.g., zooming on mobile devices)
			this.rotation = (this.rotation + 90) % 180
			this.htmlObject.style.transform = `rotate(${this.rotation}deg)`
		}

		handleMouseDown(event) {
			this.htmlObject.style.left = `${event.clientX - OFFSET + this.rotationOffsetLeft()}px`
			this.htmlObject.style.top = `${event.clientY - OFFSET_X - OFFSET_Y + this.rotationOffsetTop()}px`

			document.addEventListener('mousemove', this.handleMouseMove)
			document.addEventListener('mouseup', this.handleMouseUp)
		}

		handleMouseMove(event) {
			this.htmlObject.style.left = `${event.clientX - OFFSET + this.rotationOffsetLeft()}px`
			this.htmlObject.style.top = `${event.clientY - OFFSET_X - OFFSET_Y + this.rotationOffsetTop()}px`
		}

		handleMouseUp(event) {
			function distance(x1, y1, x2, y2) {
				return Math.sqrt(Math.pow(x1 - x2, 2) + Math.pow(y1 - y2, 2))
			}
			audioManager.addToQueue('sounds/enter_coordinates_first_letter.mp3')
			audioManager.addToQueue('sounds/enter_coordinates_second_number.mp3')

			document.removeEventListener('mousemove', this.handleMouseMove)
			document.removeEventListener('mouseup', this.handleMouseUp)

			let minDistance = Infinity
			let closestRectCell
			let closestCell

			cells.forEach((cell) => {
				const cellRect = cell.getBoundingClientRect()
				const currentDistance = distance(event.clientX, event.clientY, cellRect.left + cellRect.width / 2, cellRect.top + cellRect.height / 2)

				if (currentDistance < minDistance) {
					minDistance = currentDistance
					closestRectCell = cellRect
					closestCell = cell
				}
			})

			this.htmlObject.style.left = `${closestRectCell.left - 543 + this.rotationOffsetLeft()}px`
			this.htmlObject.style.top = `${closestRectCell.top - OFFSET_Y + this.rotationOffsetTop()}px`
			this.htmlObject.coord = Array.from({ length: this.length }, (_, i) => [
				parseInt(closestCell.dataset.row) + i,
				parseInt(closestCell.dataset.col)
			])
		}
	}

	const fleet = [new Ship(2, '.shipTwo'), new Ship(3, '.shipThreeB'), new Ship(3, '.shipThreeA'), new Ship(4, '.shipFour'), new Ship(5, '.shipFive')]
	const cells = document.querySelectorAll('.cell')
	const opponentCells = document.querySelectorAll('.opponentCell')
	const finishedButton = document.getElementById('finishedPlacement')
	finishedButton.addEventListener('click', sendPostionsToServer)

	const audioManager = new AudioManager()



	window.onload = function () {

		console.log("test")
		var currentPath = window.location.pathname;
		var pathSegments = currentPath.split('/');
		console.log(pathSegments)

		if (currentPath === "/") {
			fetch(`http://${BACKEND_URL}/new_game/`)
				.then(response => response.json())
				.then(data => {
					game_id = data.game_id
					player_id = data.player_id
					document.getElementById('player_b_url').innerHTML = `Player B URL =  <a href="http://${FRONTEND_URL}/${game_id}">URL for player b</a>`
				})
				.catch(error => console.error('An error occurred:', error));
		} else if (pathSegments[1] != "") {
			fetch(`http://${BACKEND_URL}/join_game/${pathSegments[1]}`)
				.then(response => response.json())
				.then(data => {
					game_id = data.game_id
					player_id = data.player_id
				})
				.catch(error => console.error('An error occurred:', error));
		} else {
			console.log(`unkown path provided ${currentPath}`)
		}
	};




	async function sendPostionsToServer() {
		audioManager.addToQueue(audioManager.sounds.newGame)
		let position= fleet.map(ship => ship.htmlObject.coord)

		let data  = {"player_id": player_id, "position": position }
		const url = `http://${BACKEND_URL}/position/${game_id}`

		console.log(data)
		try {
			const response = await fetch(url, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify( data)
			})

			if (response.ok) {
				console.log(response)
				handleSuccessfullPositionPost()
			} else {
				console.log('Error while sending JSON. Server response:', response.status)
			}
		} catch (error) {
			console.error('Error while sending JSON:', error)
		}

		const eventSource = new EventSource(`http://${BACKEND_URL}/events/${game_id}/${player_id}`)
		eventSource.addEventListener('message', handleServerEvent)
	}

	function handleSuccessfullPositionPost() {
		finishedButton.style.display = 'none'
		opponentCells.forEach((cell) => {
			cell.addEventListener('click', handleCellClickOnOpponentCells)
		})
	}

	function handleServerEvent(event) {
		audioManager.addToQueue(audioManager.sounds.shot)
		const result = JSON.parse(event.data)
		console.log('JSON-Event vom Backend-Server empfangen:', result)

		if (result.hit) {
			audioManager.addToQueue(audioManager.sounds.hit)
			audioManager.addToQueue(audioManager.sounds.ownHit)
		} else {
			audioManager.addToQueue(audioManager.sounds.miss)
		}
		if (result.sunk) {
			audioManager.addToQueue(audioManager.sounds.lostOwnShip)
			for (let i = 1; i <= parseInt(result.length); i++) {
				audioManager.addToQueue(audioManager.sounds.lostPoint)
			}
			audioManager.addToQueue(audioManager.sounds.ownSunk)
		}
		if (result.fleet_destroyed) {
			audioManager.addToQueue(audioManager.sounds.ending)
		}
		const ownCells = document.querySelectorAll('.cell')
		ownCells.forEach(function (cell) {
			if (result.shot[0] == cell.dataset.row && result.shot[1] == cell.dataset.col) {
				cell.style.backgroundColor = 'black'
				cell.style.zIndex = 100
			}
		})
	}

	async function handleCellClickOnOpponentCells(event) {
		const { row, col } = event.target.dataset
		const jsonData = { player_id: player_id, x: parseInt(row), y: parseInt(col)}

		try {
			const url = `http://${BACKEND_URL}/shot/${game_id}`
			const response = await fetch(url, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(jsonData)
			})

			if (response.ok) {
				handleShot(response, event.target)
			} else {
				audioManager.addToQueue(audioManager.sounds.error)
				console.log('Error while sending JSON. Server response:', response.status)
			}
		} catch (error) {
			console.error('Error while sending JSON:', error)
		}
	}

	async function handleShot(response, clickedCell) {
		audioManager.addToQueue(audioManager.sounds.shot)
		const result = await response.json()
		console.log(result)
		if (result.hit) {
			clickedCell.style.backgroundColor = 'red'
			audioManager.addToQueue(audioManager.sounds.hit)
			audioManager.addToQueue(audioManager.sounds.enemyHit)
		} else {
			clickedCell.style.backgroundColor = 'blue'
			audioManager.addToQueue('sounds/shot_2_water.mp3')
		}
		if (result.sunk) {
			audioManager.addToQueue(audioManager.sounds.sinking)
			for (let i = 1; i <= parseInt(result.length); i++) {
				audioManager.addToQueue(audioManager.sounds.lostPoint)
			}
			audioManager.addToQueue(audioManager.sounds.sunk)
		}
		if (result.fleet_destroyed) {
			audioManager.addToQueue(audioManager.sounds.ending)
		}
	}
})
