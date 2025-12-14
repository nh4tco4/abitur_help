import telebot
import requests
from telebot import types
from bot_token import BOT_TOKEN
bot = telebot.TeleBot(BOT_TOKEN)
from question_and_answers import dict_sections_questions_answers
data = dict_sections_questions_answers
from site_mai_links import dict_names_links
names_links = dict_names_links
DEBUG = False

@bot.message_handler(commands = ['start'], func=lambda message: message.chat.type == "private")
def start(message):
    markup = types.InlineKeyboardMarkup()
    for name in names_links:
        button = types.InlineKeyboardButton(text=name, url=names_links[name])
        markup.add(button)
    bot.send_message(message.from_user.id, 'По кнопке ниже можно перейти на сайт', reply_markup = markup)
    keyboard = types.ReplyKeyboardMarkup(row_width=3, resize_keyboard=True)
    if message.chat.type == "private":
        for section in data:
            button = types.KeyboardButton(section)
            keyboard.add(button)
        bot.reply_to(message, 'Выберите раздел или напишите ваш запрос.', reply_markup=keyboard)


@bot.message_handler(func=lambda message: message.chat.type == "group" or message.chat.type == "supergroup")
def handle_question(message):
    payload = {"query": message.text}
    try:
        response_server = requests.post(
            "http://backend:8080/api/search",
            json=payload,
            timeout=8
        )
        if response_server.ok:
            response_data = response_server.json()
            if response_data.get("status") == "found":
                reply = f"{response_data['question']}\n{response_data['answer']}"
                if response_data.get("similar_questions"):
                    reply += "\n\nПохожие вопросы:\n" + "\n".join(response_data["similar_questions"])
                bot.reply_to(message, reply)
            else:
                reply = response_data.get("message", "Нет совпадений.")
                if response_data.get("suggestions"):
                    reply += "\n\nПредложения:\n" + "\n".join(response_data["suggestions"])
                bot.reply_to(message, reply)
        else:
            bot.reply_to(message, "Сервис временно недоступен, попробуйте позже.")
    except requests.exceptions.RequestException:
        bot.reply_to(message, "Не могу связаться с сервером поиска. Попробуйте позже.")


@bot.message_handler(func=lambda message: message.chat.type == "private")
def handle_message(message):
    if message.text in data:
        section = message.text
        keyboard = types.ReplyKeyboardMarkup(row_width=3, resize_keyboard=True)
        back_button = types.KeyboardButton("⬅️ Назад к разделам")
        keyboard.add(back_button)
        for question in data[section]:
            button = types.KeyboardButton(question)
            keyboard.add(button)
        bot.reply_to(message, 'Вы выбрали раздел "' + section + '", теперь выберите или запишите ваш вопрос', reply_markup=keyboard)

    elif message.text == '⬅️ Назад к разделам':
        keyboard = types.ReplyKeyboardMarkup(row_width=3, resize_keyboard=True)
        for section in data:
            button = types.KeyboardButton(section)
            keyboard.add(button)
        bot.reply_to(message, 'Выберите раздел или напишите ваш запрос.', reply_markup=keyboard)
    else:
        found = False
        for section, questions in data.items():
            if message.text in questions:
                bot.reply_to(message, questions[message.text])
                found = True
                break
        if not found:
            payload = {"query": message.text}
            try:
                response_server = requests.post(
                    "http://backend:8080/api/search",
                    json=payload,
                    timeout=8
                )
                if response_server.ok:
                    response_data = response_server.json()
                    if response_data.get("status") == "found":
                        reply = f"{response_data['question']}\n{response_data['answer']}"
                        if response_data.get("similar_questions"):
                            reply += "\n\nПохожие вопросы:\n" + "\n".join(response_data["similar_questions"])
                        bot.reply_to(message, reply)
                    else:
                        reply = response_data.get("message", "Нет совпадений.")
                        if response_data.get("suggestions"):
                            reply += "\n\nПредложения:\n" + "\n".join(response_data["suggestions"])
                        bot.reply_to(message, reply)
                else:
                    bot.reply_to(message, "Сервис временно недоступен, попробуйте позже.")
            except requests.exceptions.RequestException:
                bot.reply_to(message, "Не могу связаться с сервером поиска. Попробуйте позже.")


bot.polling(none_stop=True, interval=0)
