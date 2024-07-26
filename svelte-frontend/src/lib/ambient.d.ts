type Question = {
  question_type: string;
  question_text: string | null;
  value: number;
  answer: string | null;
  won_user_id: any | null;
};

type Category = {
  title: string;
  questions: Question[];
};

type GameData = {
  creator: string;
  categories: Category[];
  current: null | string;
};