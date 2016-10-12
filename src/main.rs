mod article_iterator;

fn main() {
    let mut articles_read = 0;
    for article in article_iterator::ArticleIterator::new("wikipedia.dump.bz2") {
        if !article.starts_with("#REDIRECT") {
            articles_read += 1;
            println!("article {}: {}\n-----------", articles_read, article);
        }
    }
}

