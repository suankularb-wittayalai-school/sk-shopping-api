use lettre::{
    error::Error, message::header::ContentType, transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use lettre_email::Email;

use crate::models::{item::Item, order::Order};

pub fn send_invoice_email(credential: &Credentials, order: Order) -> Result<(), Error> {
    // let (email_address, ref_id) = match order {
    //     Order::IdOnly(order) => {
    //         return Err(Error::MissingTo);
    //     },
    //     Order::Compact(order) => {
    //         return Err(Error::MissingTo);
    //     },
    //     Order::Default(order) => (order.contact_email, order.ref_id),
    //     Order::Detailed(order) => (order.contact_email, order.ref_id),
    // };

    // email content
    // reciever name
    // items and amount
    // total price
    // delivery type
    // pickup location
    // payment method

    let (
        email_address,
        ref_id,
        receiver_name,
        items,
        total_price,
        delivery_type,
        pickup_location,
        payment_method,
    ) = match order {
        Order::IdOnly(order) => {
            return Err(Error::MissingTo);
        }
        Order::Compact(order) => {
            return Err(Error::MissingTo);
        }
        Order::Default(order) => (
            order.contact_email,
            order.ref_id,
            order.receiver_name,
            order.items,
            order.total_price,
            order.delivery_type,
            order.pickup_location,
            order.payment_method,
        ),
        Order::Detailed(order) => (
            order.contact_email,
            order.ref_id,
            order.receiver_name,
            order.items,
            order.total_price,
            order.delivery_type,
            order.pickup_location,
            order.payment_method,
        ),
    };

    let html_content = format!(
        r#"
        <html>
            <head>
                <title>Invoice for order {}</title>
            </head>
            <body>
                <h1>Invoice for order {}</h1>
                <p>Dear {}</p>
                <p>Thank you for your order. Here is your invoice.</p>
                <table>
                    <thead>
                        <tr>
                            <th>Item</th>
                            <th>Amount</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
                <p>Total price: {}</p>
                <p>Delivery type: {}</p>
                <p>Pickup location: {}</p>
                <p>Payment method: {}</p>
            </body>
        </html>
        "#,
        ref_id,
        ref_id,
        receiver_name,
        items
            .iter()
            .map(|item| {
                let (item_name, price) = match &item.item {
                    Item::IdOnly(_) => {
                        return Err(Error::MissingTo);
                    }
                    Item::Compact(item) => (
                        &item.name,
                        std::cmp::min(item.price, item.discounted_price.unwrap_or(item.price)),
                    ),
                    Item::Default(item) => (
                        &item.name,
                        std::cmp::min(item.price, item.discounted_price.unwrap_or(item.price)),
                    ),
                    Item::Detailed(item) => (
                        &item.name,
                        std::cmp::min(item.price, item.discounted_price.unwrap_or(item.price)),
                    ),
                };
                Ok(format!(
                    r#"
                <tr>
                    <td>{}</td>
                    <td>{}</td>
                </tr>
                "#,
                    item_name, item.amount
                ))
            })
            .collect::<Result<String, Error>>()?,
        total_price,
        delivery_type,
        pickup_location.unwrap_or(vec![]).join(", "),
        payment_method
    );

    let to = format!("{} <{}>", receiver_name, email_address).parse();

    let to = match to {
        Ok(to) => to,
        Err(_) => {
            return Err(Error::MissingTo);
        }
    };

    let from = "คณะกรรมการนักเรียน <kornor@sk.ac.th>".parse();

    let from = match from {
        Ok(from) => from,
        Err(_) => {
            return Err(Error::MissingFrom);
        }
    };

    let email = Message::builder()
        .to(to)
        .from(from)
        .subject(format!("Invoice for order {}", ref_id))
        .header(ContentType::TEXT_HTML)
        .body(html_content);

    let email = match email {
        Ok(email) => email,
        Err(_) => {
            return Err(Error::MissingFrom);
        }
    };

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(credential.clone())
        .build();

    mailer.send(&email);

    Ok(())
}
